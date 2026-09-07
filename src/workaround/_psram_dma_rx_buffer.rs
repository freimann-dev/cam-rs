use esp_hal::dma::{
    BurstConfig, DmaDescriptor, DmaDescriptorFlags, DmaRxBuffer, ExternalBurstConfig,
    InternalBurstConfig, Preparation, TransferDirection,
};

/// Структура-мост для легального проброса PSRAM в LCD_CAM DMA.
///
/// Позволяет обойти временное софтверное ограничение `esp-hal 1.1.2`,
/// принудительно переключая канал GDMA в аппаратный режим EDMA (работа с PSRAM).
pub struct PsramDmaRxBuffer {
    // Храним сырой указатель на начало цепочки дескрипторов во внутренней SRAM,
    // чтобы метод `from_view` мог безопасно восстановить структуру после кадра.
    desc_ptr: *mut DmaDescriptor,
    desc_count: usize,
    chunk_size: usize,
    pub psram_slice: &'static mut [u8],
}

unsafe impl DmaRxBuffer for PsramDmaRxBuffer {
    type View = *mut DmaDescriptor; // В качестве View передаем указатель на дескрипторы
    type Final = Self;

    fn prepare(&mut self) -> Preparation {
        let mut current_psram_ptr = self.psram_slice.as_mut_ptr();

        unsafe {
            for i in 0..self.desc_count {
                let desc = &mut *self.desc_ptr.add(i);
                let mut flags = DmaDescriptorFlags(0);
                flags.set_size(self.chunk_size as u16);
                flags.set_length(0);
                flags.set_suc_eof(false);

                desc.flags = flags;
                desc.set_owner(esp_hal::dma::Owner::Dma);
                desc.buffer = current_psram_ptr;

                if i < (self.desc_count - 1) {
                    desc.next = self.desc_ptr.add(i + 1);
                    current_psram_ptr = current_psram_ptr.add(self.chunk_size);
                } else {
                    desc.next = core::ptr::null_mut();
                    desc.set_suc_eof(true); // Конец кадра
                }
            }
        }

        // Принудительно включаем аппаратную поддержку PSRAM в EDMA
        Preparation {
            start: self.desc_ptr,
            direction: TransferDirection::In,
            accesses_psram: true,
            burst_transfer: BurstConfig {
                external_memory: ExternalBurstConfig::default(),
                internal_memory: InternalBurstConfig::Disabled,
            },
            check_owner: Some(true),
            auto_write_back: true,
        }
    }

    fn into_view(self) -> Self::View {
        self.desc_ptr
    }

    fn from_view(view: Self::View) -> Self::Final {
        // Метод вызывается ХАЛом автоматически при завершении `.wait()`.
        // В этой точке кэш процессора L1 уже автоматически инвалидирован!
        unsafe {
            // Восстанавливаем исходные параметры буфера
            let desc_count = 128; // Эти константы должны соответствовать вашему дефолтному конфигу
            let chunk_size = 4032;

            PsramDmaRxBuffer {
                desc_ptr: view,
                desc_count,
                chunk_size,
                psram_slice: core::slice::from_raw_parts_mut(
                    0x3C00_0000 as *mut u8,
                    chunk_size * desc_count,
                ),
            }
        }
    }
}

/// Традиционный макрос для быстрого развертывания PSRAM DMA буфера.
/// Нарезает дескрипторы в SRAM, а данные мапит на физическое окно PSRAM.
#[macro_export]
macro_rules! psram_dma_rx_buffer {
    ($desc_count:expr, $chunk_size:expr) => {{
        // Статический массив дескрипторов создается во внутренней SRAM
        static mut DESCRIPTORS: [esp_hal::dma::DmaDescriptor; $desc_count] =
            [esp_hal::dma::DmaDescriptor::EMPTY; $desc_count];

        // Мапим срез на физический адрес PSRAM
        let psram_mem = unsafe {
            core::slice::from_raw_parts_mut(
                0x3C00_0000 as *mut u8,
                $chunk_size * $desc_count
            )
        };

        // Путь указывает на структуру внутри папки workaround вашего крейта
        $crate::workaround::psram_dma_rx_buffer::PsramDmaRxBuffer {
            desc_ptr: unsafe { DESCRIPTORS.as_mut_ptr() },
            desc_count: $desc_count,
            chunk_size: $chunk_size,
            psram_slice: psram_mem,
        }
    }};
}
