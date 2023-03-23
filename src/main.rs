extern crate image;
extern crate rayon;
extern crate rand;

use std::fs::{self};
use rand::Rng;
use tqdm::Iter;
use std::path::{Path};
use std::time::{Instant, Duration};
use image::imageops::{FilterType};
use image::{RgbImage, ImageError, SubImage, DynamicImage};
use rayon::prelude::*;
use std::sync::Mutex;

fn main() {
    
    let imagen_objetivo = "C:/Users/Ian/Downloads/gato.jpg";

    let imagen_final_result = armar_imagen_objetivo(imagen_objetivo);
    
    let random_value = rand::thread_rng().gen_range(0..100).to_string();
    
    let image_name = "imagenPrueba".to_owned() + &random_value + ".png";
    println!("{}", image_name);

    match imagen_final_result {
        Ok(imagen) => imagen.save(image_name).unwrap(),
        Err(_) => println!("hubo error."),
    };

    println!("listo");
}

fn load_images_from_dir(dir: &Path) -> Vec<RgbImage> {

    // Recorremos todos los archivos y cargamos las imágenes que encontremos
    let paths = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    let images = paths.par_iter().map(|path| image::open(path).unwrap().to_rgb8()).collect();

    return images
}

fn armar_imagen_objetivo(imagen: &str) -> Result<RgbImage, ImageError> {

    println!("preparo.");

    let filas: u32 = 60;
    let columnas: u32 = 60;

    let ancho: u32 = 12000;
    let alto: u32= 12000;

    println!("ancho / columnas = {}, alto / filas = {}", ancho / columnas, alto / filas);
    
    let image = image::open(imagen).unwrap().to_rgb8();
    
    let image_rgb_data = load_images_from_dir(Path::new("D:/movie posters"));

    let ancho_seccion: u32 = ancho / columnas;
    let alto_seccion: u32 = alto / filas;

    let mut img_final: RgbImage = image::ImageBuffer::new(ancho, alto);

    let mut principio_seccion_x: u32;
    let mut principio_seccion_y: u32;

    println!("empiezo.");

    let start2 = Instant::now();

    let mut tiempo_comparar = 0;
    let mut tiempo_cuentas = 0;

    for seccion in (0..(filas * columnas)).into_iter().tqdm() {
        
        if seccion > 0 {
            principio_seccion_x = (seccion * ancho_seccion) % ancho;
            principio_seccion_y = (seccion / filas) * alto_seccion;
        } else {
            principio_seccion_x = 0;
            principio_seccion_y = 0;
        }

        let principio_seccion_x_base: f32 = ((principio_seccion_x as f32 / img_final.width() as f32) * image.width() as f32).into();
        let principio_seccion_y_base: f32 = ((principio_seccion_y as f32 / img_final.height() as f32) * image.height() as f32).into();
         
        let ancho_seccion_base: f32 = ((ancho_seccion as f32 / img_final.width() as f32) * image.width() as f32).into();
        let alto_seccion_base: f32 = ((alto_seccion as f32 / img_final.height() as f32) * image.height() as f32).into();

        let subimagen: RgbImage = SubImage::new(&image, principio_seccion_x_base as u32, principio_seccion_y_base as u32, ancho_seccion_base as u32, alto_seccion_base as u32).to_image();
        let imagen_seccion: RgbImage = imagen_mas_cercana_exacto(&subimagen, &image_rgb_data);

        let resized_image_seccion: RgbImage = image::imageops::resize(&imagen_seccion, ancho / columnas, alto / filas, FilterType::Nearest);

        image::imageops::overlay(&mut img_final, &resized_image_seccion, principio_seccion_x.into(), principio_seccion_y.into());
        
    }

    let duration2: Duration = start2.elapsed();

    println!("terminó de cocinarse en {} segundos", duration2.as_secs());
  
    return Ok(img_final)
}

fn imagen_mas_cercana_exacto(imagen_seccion: &RgbImage, images:& Vec<RgbImage>) -> RgbImage {
    // Función que encuentra la imagen que más se parece a una imagen de referencia 

    // Use a Mutex to protect the most_similar_image variable
    let most_similar_image: Mutex<image::ImageBuffer<image::Rgb<u8>, Vec<u8>>> = Mutex::new(image::ImageBuffer::new(0,0));
    let most_similarity: Mutex<f64> = Mutex::new(0.0);

        // Process each chunk in a separate thread
    images.into_par_iter().for_each(|image| {
        let start_cuentas = Instant::now();

        let similarity = compare_images(&imagen_seccion, &image);
        tiempo_cuentas += start_cuentas.elapsed().as_millis();
        // Update the most similar image if necessary
        let mut most_similar_image = most_similar_image.lock().unwrap();
        let mut most_similarity = most_similarity.lock().unwrap();
        if similarity > *most_similarity {
            *most_similar_image = image.clone();
            *most_similarity = similarity;
        }
    });
    
    return (*most_similar_image.lock().unwrap()).clone();
}

// Función que compara dos imágenes pixel por pixel y devuelve su parecido en términos de porcentaje
fn compare_images(image1: &RgbImage, image2: &RgbImage) -> f64 {
    let mut num_pixels_equal = 0;
    // Recorremos ambas imágenes pixel por pixel y comparamos los valores de cada uno
    for (x, y, pixel1) in image1.enumerate_pixels() {

        let pixel_x: f32 = ((x as f32 / image1.width() as f32) * image2.width() as f32).into();
        let pixel_y: f32 = ((y as f32 / image1.height() as f32) * image2.height() as f32).into();
 
        let pixel2 = image2.get_pixel(pixel_x as u32, pixel_y as u32);

        // println!("pixel_x: {}, pixel_y: {}, pixel2: {}, pixel1 {}", pixel_x, pixel_y, pixel2.0[0], pixel1.0[0]);
        if color_close_enough(pixel1, pixel2, 20.0) {
            num_pixels_equal += 1;
        }
    }

    // Calculamos el porcentaje de parecido entre las imágenes
    let num_pixels = (image1.width() * image1.height()) as f64;
    let similarity = num_pixels_equal as f64 / num_pixels;
    return similarity
}

fn color_close_enough(rgb1: &image::Rgb<u8>, rgb2: &image::Rgb<u8>, closeness: f32) -> bool { 
  
    let red_diff = (rgb1.0[0] as f32 - rgb2.0[0] as f32).abs() * 0.299;
    let green_diff = (rgb1.0[1] as f32 - rgb2.0[1] as f32).abs() * 0.587;
    let blue_diff = (rgb1.0[2] as f32 - rgb2.0[2] as f32).abs() * 0.114;

    let distance = (red_diff + green_diff + blue_diff) as f32;

    return distance < closeness 
}
