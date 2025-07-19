pub mod ui;
pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod network;

// Unix-based UI modules
pub mod unix_cpu;
pub mod unix_gpu;
pub mod unix_npu;
pub mod unix_rga;

pub use ui::Ui;
pub use ui::UiMode;
