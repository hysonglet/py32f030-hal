//！ embassy时钟移植接口
//！ 提供embassy时钟心跳
//!  用户无需关注这个模块的内部接口

mod time_driver_systick;

pub fn init() {
    critical_section::with(time_driver_systick::init);
}
