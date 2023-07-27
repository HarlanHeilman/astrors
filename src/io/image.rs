use std::fs::File;
use std::io::Read;

use crate::io::header::Header;

use rayon::prelude::*;
use ndarray::ArrayBase;
use byteorder::{ByteOrder, NativeEndian};

use std::io::{Write, BufWriter};

use crate::io::aux::{
    get_shape,
    pre_bytes_to_f64_vec,
    pre_bytes_to_f32_vec,
    pre_bytes_to_u8_vec,
    pre_bytes_to_i16_vec,
    pre_bytes_to_i32_vec, 
    vec_to_ndarray, 
    DataType
};
use ndarray::ArrayD;

pub struct Data {
    u8: Option<ArrayD<u8>>,
    i16: Option<ArrayD<i16>>,
    i32: Option<ArrayD<i32>>,
    f32: Option<ArrayD<f32>>,
    f64: Option<ArrayD<f64>>,
    dtype: DataType,
}

use std::fmt;
impl fmt::Debug for Data {
    
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.dtype {
            DataType::u8 => {
                write!(f, "Data: {:?}, dtype {:?}", self.u8, self.dtype)
            }
            DataType::i16 => {
                write!(f, "Data: {:?}, dtype {}", self.i16, self.dtype)
            }
            DataType::i32 => {
                write!(f, "Data: {:?}, dtype {}", self.i32, self.dtype)
            }
            DataType::f32 => {
                write!(f, "Data: {:?}, dtype {}", self.f32, self.dtype)
            }
            DataType::f64 => {
                write!(f, "Data: {:?}, dtype {}", self.f64, self.dtype)
            }
        }
    }
}


impl Data {
    pub fn new() -> Data {
        Data {
            u8: None,
            i16: None,
            i32: None,
            f32: None,
            f64: None,
            dtype: DataType::u8,
        }
    }
    pub fn pretty_print(&self) {
        match self.dtype {
            DataType::u8 => {
                println!("Data: {:?}", self.u8);
            },
            DataType::i16 => {
                println!("Data: {:?}", self.i16);
            },
            DataType::i32 => {
                println!("Data: {:?}", self.i32);
            },
            DataType::f32 => {
                println!("Data: {:?}", self.f32);
            },
            DataType::f64 => {
                println!("Data: {:?}", self.f64);
            },
        }
        println!("Data type: {:?}", self.dtype);
    }
}

impl Data {

    pub fn read_from_buffer(f: &mut File, header: &mut Header) -> Result<Data, std::io::Error>  {
        let _naxis: usize = header.parse_header_value("NAXIS")?;
    
        let bitpix : i32 = header.parse_header_value("BITPIX")?;
        let shape = get_shape(header)?;

        // Get data type from BITPIX
        let dtype = DataType::from_bitpix(bitpix).unwrap();
        let dtype_bytes = dtype.nbytes();
        // println!("nbytes: {:?}", nbytes); 

        let total_bytes = shape.iter().fold(1, |acc, x| acc * x) * dtype_bytes;
        let mut databuf = vec![0; total_bytes]; 
        let _ = f.read(&mut databuf)?;

        // Read until the end of the current FITS block
        let remainder = total_bytes % 2880;
        if remainder != 0 {
            let mut padding = vec![0; 2880 - remainder];
            let _ = f.read(&mut padding)?;
            // println!("Padding: {:?}", padding.len());
        }

        Data::image_buffer_to_ndarray(databuf, shape, dtype) 
    }

    pub fn image_buffer_to_ndarray(databuf: Vec<u8>, shape: Vec<usize>, dtype: DataType) -> Result<Data, std::io::Error>  {
        match dtype {
            DataType::u8 => {
                let mut vect: Vec<u8> = vec![0; databuf.len() / 1];
                pre_bytes_to_u8_vec(databuf, &mut vect);

                let ndarray = vec_to_ndarray(vect, shape);
                
                let mut data = Data::new();
                data.u8 = Some(ndarray);      
                data.dtype = DataType::u8;


                Ok(data)
            },
            DataType::i16 => {
                let mut vect: Vec<i16> = vec![0; databuf.len() / 2];
                pre_bytes_to_i16_vec(databuf, &mut vect);
                
                let ndarray = vec_to_ndarray(vect, shape);

                let mut data = Data::new();
                data.i16 = Some(ndarray);      
                data.dtype = DataType::i16;

                Ok(data)
            },
            DataType::i32 => {
                let mut vect: Vec<i32> = vec![0; databuf.len() / 4];
                pre_bytes_to_i32_vec(databuf, &mut vect);
                
                let ndarray = vec_to_ndarray(vect, shape);

                let mut data = Data::new();
                data.i32 = Some(ndarray);      
                data.dtype = DataType::i32;

                Ok(data)
            },
            DataType::f32 => {
                let mut vect: Vec<f32> = vec![0.0; databuf.len() / 4];
                pre_bytes_to_f32_vec(databuf, &mut vect);

                let ndarray = vec_to_ndarray(vect, shape);
                
                let mut data = Data::new();
                data.f32 = Some(ndarray);      
                data.dtype = DataType::f32;

                Ok(data)
            },
                DataType::f64 => {
                let mut vect: Vec<f64> = vec![0.0; databuf.len() / 8];
                pre_bytes_to_f64_vec(databuf, &mut vect);

                let ndarray = vec_to_ndarray(vect, shape);

                let mut data = Data::new();
                data.f64 = Some(ndarray);      
                data.dtype = DataType::f64;

                Ok(data)
            },
            _ => {
                panic!("Not implemented");
            }   
        }
    }

    pub fn ndarray_to_buffer_parallel(&self) -> Vec<u8> {
        match self.dtype {
            DataType::u8 => {
                let mut vect = self.u8.as_ref().unwrap().clone().into_raw_vec();
                vect.par_iter().flat_map(|&item| item.to_ne_bytes().to_vec()).collect::<Vec<u8>>()
            },
            DataType::i16 => {
                let mut vect = self.i16.as_ref().unwrap().clone().into_raw_vec();
                vect.par_iter().flat_map(|&item| item.to_ne_bytes().to_vec()).collect::<Vec<u8>>()
            },
            DataType::i32 => {
                let mut vect = self.i32.as_ref().unwrap().clone().into_raw_vec();
                vect.par_iter().flat_map(|&item| item.to_ne_bytes().to_vec()).collect::<Vec<u8>>()
            },
            DataType::f32 => {
                let mut vect = self.f32.as_ref().unwrap().clone().into_raw_vec();
                vect.par_iter().flat_map(|&item| item.to_ne_bytes().to_vec()).collect::<Vec<u8>>()
            },
            DataType::f64 => {
                let mut vect = self.f64.as_ref().unwrap().clone().into_raw_vec();
                vect.par_iter().flat_map(|&item| item.to_ne_bytes().to_vec()).collect::<Vec<u8>>()
            },
            _ => {
                panic!("Not implemented");
            }
        }
    }

    pub fn write_to_buffer(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        let mut buffer = self.ndarray_to_buffer_parallel();
        let remainder = buffer.len() % 2880;
        if remainder != 0 {
            let padding = vec![0; 2880 - remainder];
            buffer.extend(padding);
        }
        writer.write_all(&buffer)?;
        Ok(())
    }

    pub fn ndarray_to_buffer<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        let mut writer = BufWriter::new(writer);
        let mut bytes_written = 0;
        match self.dtype {
            DataType::u8 => {
                if let Some(ndarray) = &self.u8 {
                    for &item in ndarray.iter() {
                        let bytes: [u8; 1] = item.to_be_bytes();
                        writer.write_all(&bytes)?;
                        bytes_written += bytes.len();
                    }
                }
            },
            DataType::i16 => {
                if let Some(ndarray) = &self.i16 {
                    for &item in ndarray.iter() {
                        let bytes: [u8; 2] = item.to_be_bytes();
                        writer.write_all(&bytes)?;
                        bytes_written += bytes.len();
                    }
                }
            },
            DataType::i32 => {
                if let Some(ndarray) = &self.i32 {
                    for &item in ndarray.iter() {
                        let bytes: [u8; 4] = item.to_be_bytes();
                        writer.write_all(&bytes)?;
                        bytes_written += bytes.len();
                    }
                }
            },
            DataType::f32 => {
                if let Some(ndarray) = &self.f32 {
                    for &item in ndarray.iter() {
                        let bytes: [u8; 4] = f32::to_be_bytes(item);
                        writer.write_all(&bytes)?;
                        bytes_written += bytes.len();
                    }
                }
            },
            DataType::f64 => {
                if let Some(ndarray) = &self.f64 {
                    for &item in ndarray.iter() {
                        let bytes: [u8; 8] = f64::to_be_bytes(item);
                        writer.write_all(&bytes)?;
                        bytes_written += bytes.len();
                    }
                }
            },
            
            _ => {
                panic!("Not implemented");
            }
        }
        let remainder = bytes_written % 2880;
        if remainder != 0 {
            let padding = vec![0; 2880 - remainder];
            bytes_written += padding.len(); // increment the counter
            writer.write_all(&padding)?;
        }
        writer.flush()
    }

}

#[test]
fn read_image_test() -> std::io::Result<()>{
    // crate::fits_io::read_file();
    use crate::{GLOBAL_FILE_NAME, WRITE_FILE};

    use std::time::Instant;
    let now = Instant::now();

    rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();

    use std::fs::File;
    // let mut f = File::open("./testdata/test.fits")?
    let mut f: File = File::open(GLOBAL_FILE_NAME.as_str())?;

    let mut header = crate::io::header::Header::new();
    header.read_from_file(&mut f)?;
    header.pretty_print();

    use std::io::Write;
    // let mut file = File::create(WRITE_FILE.as_str())?;
    let mut file = File::create("output.fits")?;
    header.write_to(&mut file)?;
    
    file.flush()?;

    let mut data = crate::io::image::Data::read_from_buffer(&mut f, &mut header)?;

    

    println!("Data: {:?}", data);
    

    if let Some(ndarray) = &data.f32 {
        println!("Data Mean: {:?}", ndarray.mean());
    }
    data.f32 = None;
    data.ndarray_to_buffer(&mut file);
    
    use rayon::prelude::*;
    println!("{} threads", rayon::current_num_threads());

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(())
}