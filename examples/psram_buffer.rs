#![no_std]
#![no_main]
#![feature(slice_ptr_get)]

use esp_backtrace as _;
use esp_hal::{
    dma::{
        BurstConfig, DmaDescriptor, DmaDescriptorFlags, DmaRxBuffer, ExternalBurstConfig,
        InternalBurstConfig, Preparation, TransferDirection,
    },
    lcd_cam::{
        LcdCam,
        cam::{Camera, Config},
    },
    main,
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

// 1 дескриптор обрабатывает 4032 байта (кратно 64 байтам строки кэша Octal)
const CHUNK_LEN: usize = 4032;
const DESCRIPTOR_COUNT: usize = 128; // 128 дескрипторов
const BUF_LEN: usize = CHUNK_LEN * DESCRIPTOR_COUNT; // 4032 * 128 = 516 096 байт (~504 КБ!)

// Массив дескрипторов автоматически ложится во внутреннюю SRAM.
// 128 дескрипторов займут всего 128 * 12 = 1536 байт (1.5 КБ), что для SRAM неощутимо.
static mut RX_DESCRIPTORS: [DmaDescriptor; DESCRIPTOR_COUNT] =
    [DmaDescriptor::EMPTY; DESCRIPTOR_COUNT];

pub struct PsramDmaBuffer {
    descriptors: &'static mut [DmaDescriptor],
    psram_slice: &'static mut [u8],
}

pub struct PsramDmaBufferView {
    _private: (),
}

unsafe impl DmaRxBuffer for PsramDmaBuffer {
    type View = PsramDmaBufferView;
    type Final = Self;

    fn prepare(&mut self) -> Preparation {
        let mut current_psram_ptr = self.psram_slice.as_mut_ptr();

        unsafe {
            for i in 0..DESCRIPTOR_COUNT {
                let mut flags = DmaDescriptorFlags(0);
                flags.set_size(CHUNK_LEN as u16);
                flags.set_length(0);
                flags.set_suc_eof(false);

                self.descriptors[i].flags = flags;
                self.descriptors[i].set_owner(esp_hal::dma::Owner::Dma);
                self.descriptors[i].buffer = current_psram_ptr;

                if i < (DESCRIPTOR_COUNT - 1) {
                    self.descriptors[i].next = &mut self.descriptors[i + 1] as *mut DmaDescriptor;
                    current_psram_ptr = current_psram_ptr.add(CHUNK_LEN);
                } else {
                    self.descriptors[i].next = core::ptr::null_mut();
                    self.descriptors[i].set_suc_eof(true); // Конец цепочки на 504-м килобайте
                }
            }
        }

        Preparation {
            start: self.descriptors.as_mut_ptr(),
            direction: TransferDirection::In,
            accesses_psram: true, // Включаем аппаратную поддержку PSRAM в EDMA
            burst_transfer: BurstConfig {
                external_memory: ExternalBurstConfig::default(),
                internal_memory: InternalBurstConfig::Disabled,
            },
            check_owner: Some(true),
            auto_write_back: true,
        }
    }

    fn into_view(self) -> Self::View {
        PsramDmaBufferView { _private: () }
    }

    fn from_view(_view: Self::View) -> Self::Final {
        unsafe {
            PsramDmaBuffer {
                descriptors: &mut RX_DESCRIPTORS[..],
                psram_slice: core::slice::from_raw_parts_mut(0x3C00_0000 as *mut u8, BUF_LEN),
            }
        }
    }
}

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let dma_channel = peripherals.DMA_CH0;
    let lcd_cam = LcdCam::new(peripherals.LCD_CAM);
    let camera_config = Config::default();

    let camera = Camera::new(lcd_cam.cam, dma_channel, camera_config)
        .unwrap()
        .with_master_clock(peripherals.GPIO15)
        .with_pixel_clock(peripherals.GPIO6)
        .with_vsync(peripherals.GPIO13)
        .with_hsync(peripherals.GPIO7)
        .with_data0(peripherals.GPIO11)
        .with_data1(peripherals.GPIO9)
        .with_data2(peripherals.GPIO8)
        .with_data3(peripherals.GPIO10)
        .with_data4(peripherals.GPIO12)
        .with_data5(peripherals.GPIO18)
        .with_data6(peripherals.GPIO17)
        .with_data7(peripherals.GPIO16);

    println!("Camera interface ready.");

    // Инициализируем наш гигантский 504 КБ буфер в PSRAM
    let custom_buffer = unsafe {
        PsramDmaBuffer {
            descriptors: &mut RX_DESCRIPTORS[..],
            psram_slice: core::slice::from_raw_parts_mut(0x3C00_0000 as *mut u8, BUF_LEN),
        }
    };

    println!("Allocating and arming 504 KB buffer in PSRAM...");

    let _transfer = match camera.receive(custom_buffer) {
        Ok(t) => t,
        Err((_dma_err, _camera_back, _buf_back)) => {
            println!("ERROR: Hardware refused transfer setup.");
            loop {}
        }
    };

    println!("============================================================");
    println!("CRITICAL SUCCESS: 504 KB PSRAM DMA PIPELINE IS ARMED!");
    println!("Total buffer length: {} bytes", BUF_LEN);
    println!("Memory validation bypassed, EDMA is watching 0x3C00_0000.");
    println!("============================================================");

    loop {}
}
