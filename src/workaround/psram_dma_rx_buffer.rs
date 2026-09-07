use esp_hal::dma::{
    BurstConfig, DmaDescriptor, DmaDescriptorFlags, DmaRxBuffer, ExternalBurstConfig,
    InternalBurstConfig, Preparation, TransferDirection,
};

/// Generic-версия PSRAM DMA буфера.
/// - `N` — количество дескрипторов (должно соответствовать размеру статического массива)
/// - `S` — размер чанка (должен быть кратен 64 для PSRAM на ESP32-S3)
pub struct PsramDmaRxBuffer<const N: usize, const S: usize> {
    desc_ptr: *mut DmaDescriptor,
    desc_count: usize,
    chunk_size: usize,
    pub psram_slice: &'static mut [u8],
}

unsafe impl<const N: usize, const S: usize> DmaRxBuffer for PsramDmaRxBuffer<N, S> {
    // View-тип — кортеж со всеми метаданными для корректного восстановления
    type View = (*mut DmaDescriptor, usize, usize, *mut u8, usize);
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
                    desc.set_suc_eof(true);
                }
            }
        }

        Preparation {
            start: self.desc_ptr,
            direction: TransferDirection::In,
            accesses_psram: true,
            burst_transfer: BurstConfig {
                external_memory: ExternalBurstConfig::Size64, // Явно Size64 для PSRAM!
                internal_memory: InternalBurstConfig::Disabled,
            },
            check_owner: Some(true),
            auto_write_back: true,
        }
    }

    fn into_view(self) -> Self::View {
        (
            self.desc_ptr,
            self.desc_count,
            self.chunk_size,
            self.psram_slice.as_mut_ptr(),
            self.psram_slice.len(),
        )
    }

    fn from_view(view: Self::View) -> Self::Final {
        let (desc_ptr, desc_count, chunk_size, psram_ptr, psram_len) = view;
        unsafe {
            PsramDmaRxBuffer {
                desc_ptr,
                desc_count,
                chunk_size,
                psram_slice: core::slice::from_raw_parts_mut(psram_ptr, psram_len),
            }
        }
    }
}

/// Макрос, который теперь передаёт константы в тип структуры
#[macro_export]
macro_rules! psram_dma_rx_buffer {
    ($desc_count:expr, $chunk_size:expr) => {{
        // Статический массив дескрипторов с размером, заданным через generic
        static mut DESCRIPTORS: [esp_hal::dma::DmaDescriptor; $desc_count] =
            [esp_hal::dma::DmaDescriptor::EMPTY; $desc_count];

        let psram_mem = unsafe {
            core::slice::from_raw_parts_mut(
                0x3C00_0000 as *mut u8,
                $chunk_size * $desc_count,
            )
        };

        $crate::workaround::PsramDmaRxBuffer::<$desc_count, $chunk_size> {
            desc_ptr: unsafe { DESCRIPTORS.as_mut_ptr() },
            desc_count: $desc_count,
            chunk_size: $chunk_size,
            psram_slice: psram_mem,
        }
    }};
}
