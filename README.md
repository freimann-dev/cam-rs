# esp-cam-rs

Чистый Rust драйвер камер для ESP32-S3 с поддержкой DMA.

## Особенности

- ✅ 100% безопасный Rust (без `unsafe` в пользовательском коде)
- ✅ Поддержка OV5640 (OV2640 в разработке)
- ✅ DMA-захват через LCD_CAM периферию
- ✅ `no_std` совместимость
- ✅ Интеграция с `esp-hal` 1.x

## Архитектура

Драйвер состоит из трех слоев:

1. **Sensor Layer** (`sensors/`) — конкретные сенсоры (OV5640, OV2640)
2. **Capture Layer** (`capture.rs`) — низкоуровневый DMA-захват
3. **Camera Facade** (`camera.rs`) — высокоуровневый API

## Использование

### Подключение

Добавьте в `Cargo.toml`:

```toml
[dependencies]
esp-cam-rs = { path = "esp-cam-rs" }