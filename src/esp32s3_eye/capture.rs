use super::Esp32S3Eye;
use crate::BoardError;
use esp_hal::dma::DmaRxBuf;
use esp_println::println;

impl Esp32S3Eye {
    pub fn capture(&mut self, rx_buf: DmaRxBuf) -> Result<DmaRxBuf, BoardError> {
        println!("[capture] 0");
        let camera = self.camera.take().ok_or(BoardError::CaptureTimeout)?;

        println!("[capture] 1");
        let transfer = match camera.receive(rx_buf) {
            Ok(t) => t,
            Err((_dma_err, returned_camera, _returned_buf)) => {
                self.camera = Some(returned_camera);
                return Err(BoardError::InitFailed);
            }
        };

        println!("[capture] 2");
        let (dma_result, returned_camera, completed_buf) = transfer.wait();

        println!("[capture] 3");
        self.camera = Some(returned_camera);

        println!("[capture] 4");
        dma_result.map_err(|_| BoardError::DmaTransferError)?;

        // 5. ИНВАЛИДАЦИЯ D-CACHE через ассемблерные примитивы архитектуры чипа
        // let ptr = completed_buf.as_slice().as_ptr() as usize;
        // let len = completed_buf.as_slice().len();

        // unsafe {
        //     let aligned_ptr = ptr & !0x1F;
        //     let aligned_len = (len + 31) & !0x1F;

        //     xtensa_lx::cache::invalidate_dcache_range(aligned_ptr, aligned_len);
        // }

        Ok(completed_buf)
    }
}
