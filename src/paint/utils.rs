pub fn color_taple(color: u32) -> (u8,u8,u8,u8) {
    let alpha: u8 = ((color  >> 24) & 0xff)  as u8; 
    let red: u8 = ((color  >> 16) & 0xff)  as u8; 
    let green: u8  = ((color >> 8) & 0xff) as u8; 
    let blue: u8 = ((color >> 0) & 0xff) as u8; 
    (red,green,blue,alpha)
}