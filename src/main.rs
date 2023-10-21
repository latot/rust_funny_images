const one_number: usize = 1;

fn byte2number(bytes: Vec<u8>) -> isize {
    let mut ret: isize = 0;
    for i in 0..one_number {
        ret = ret + ((bytes[i] as isize) << 8);
    }
    return ret;
}

fn get_band_size(filepath: &str) -> (isize, isize) {
    let file = std::fs::File::open(filepath).unwrap();
    let mmap = unsafe { memmap::MmapOptions::new().map(&file).unwrap() };
    let mut position: usize;
    let len = mmap.len();
    if len < (one_number*2) {
        panic!("Not enough data to plot")
    }
    position = 0;
    let mut weight: isize = 0;
    let mut height: isize = 0;
    while (position + (2*one_number)) < len {
        //print!("{position}\n");
        weight = std::cmp::max(weight, byte2number(mmap[position..(position+one_number)].to_vec()));
        height = std::cmp::max(height, byte2number(mmap[(position+one_number)..(position+2*one_number)].to_vec()));
        position = position + one_number*2;
    }
    return (weight, height)
}

fn write_file(im: &mut [u64], width: isize, height: isize, filepath: &str) {
    let file = std::fs::File::open(filepath).unwrap();
    let mmap = unsafe { memmap::MmapOptions::new().map(&file).unwrap() };
    let mut position: usize;
    let len = mmap.len();
    if len < (one_number*2) {
        panic!("Not enough data to plot")
    }
    position = 0;
    while (position + (2*one_number)) < len {
        let x = byte2number(mmap[(position+0)..(position+1)].to_vec());
        let y = byte2number(mmap[(position+1)..(position+2)].to_vec());
        im[(width*y + x) as usize] = im[(width*y + x) as usize] + 1;
        position = position + 2;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::path::Path::new("test.tiff").is_file(){
        std::fs::remove_file("test.tiff")?;
    }
    let driver = gdal::DriverManager::get_driver_by_name("GTiff")?;
    let (width, height) = get_band_size("in.tiff");
    let mut ds = driver.create_with_band_type::<u64, _>("test.tiff", width, height, 1)?;
    let band = ds.rasterband(1)?;
    let mut pixel_space = 0;
    let mut line_space = 0i64;
    let mem = unsafe {
        gdal_sys::GDALGetVirtualMemAuto(
            band.c_rasterband(),
            gdal_sys::GDALRWFlag::GF_Write,
            &mut pixel_space as *mut _,
            &mut line_space as *mut _,
            std::ptr::null::<i8>() as _,
        )
    };
    let data = unsafe { gdal_sys::CPLVirtualMemGetAddr(mem) } as *mut u64;
    let len = unsafe { gdal_sys::CPLVirtualMemGetSize(mem) };
    let im = unsafe { std::slice::from_raw_parts_mut(data, len) };
    write_file(im, width, height, "in.tiff");
    unsafe { gdal_sys::CPLVirtualMemFree(mem) };
    let _ = ds.flush_cache();
    return Ok(());
}
