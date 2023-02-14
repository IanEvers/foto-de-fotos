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
    
    let imagen_objetivo = "C:/Users/Ian/Pictures/la_costa.jpg";

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

    let filas: u32 = 100;
    let columnas: u32 = 100;

    let ancho: u32 = 6400;
    let alto: u32= 6400;

    println!("ancho / columnas = {}, alto / filas = {}", ancho / columnas, alto / filas);
    
    let image = image::open(imagen).unwrap().to_rgb8();

    let image_rgb_data = load_images_from_dir(Path::new("C:/Users/Ian/Desktop/imagenes/unsplash_resized"));

    let mut image_rgb_data_resized = Vec::new();

    println!("empiezo a resizear todas las fotos");
    
    let start_resize: Instant = Instant::now();

    for image in image_rgb_data {
        let resized_image = image::imageops::resize(&image, ancho / columnas, alto / filas, FilterType::Nearest);
        image_rgb_data_resized.push(resized_image);
    }

    let duracion_resize: Duration = start_resize.elapsed();

    println!("tardó {} segundos en resizear todas las fotos", duracion_resize.as_secs());

    let start_img_resize: Instant = Instant::now();

    let dynamic_image = DynamicImage::ImageRgb8(image);

    let image_resized_dynamic: DynamicImage = DynamicImage::resize_exact(&dynamic_image, ancho, alto, image::imageops::Nearest);

    let image_resized: RgbImage = image_resized_dynamic.into_rgb8();
    
    let ancho_seccion: u32 = ancho / columnas;
    let alto_seccion: u32 = alto / filas;

    let mut img_final: RgbImage = image::ImageBuffer::new(ancho, alto);

    let mut principio_seccion_x: u32;
    let mut principio_seccion_y: u32;

    let duration = start_img_resize.elapsed();

    println!("tardé {} en resizear la imagen objetivo.", duration.as_secs());

    println!("empiezo.");

    let start2 = Instant::now();

    let mut tiempo_subimagen: u128 = 0;
    let mut tiempo_comparar_imagenes: u128 = 0;
    let mut tiempo_overlay: u128 = 0;

    for seccion in (0..(filas * columnas)).into_iter().tqdm() {

        if seccion > 0 {
            principio_seccion_x = (seccion * ancho_seccion) % ancho;
            principio_seccion_y = (seccion / filas) * alto_seccion;
        } else {
            principio_seccion_x = 0;
            principio_seccion_y = 0;
        }

        let start_subimagen: Instant = Instant::now();

        let subimagen: RgbImage = SubImage::new(&image_resized, principio_seccion_x, principio_seccion_y, ancho_seccion, alto_seccion).to_image();
        
        tiempo_subimagen += start_subimagen.elapsed().as_millis();

        let start_comparar_imagenes: Instant = Instant::now();
                
        let imagen_seccion: RgbImage = imagen_mas_cercana_exacto(&subimagen, &image_rgb_data_resized);

        tiempo_comparar_imagenes += start_comparar_imagenes.elapsed().as_millis();

        let start_overlay = Instant::now();
        
        image::imageops::overlay(&mut img_final, &imagen_seccion, principio_seccion_x.into(), principio_seccion_y.into());
        
        tiempo_overlay += start_overlay.elapsed().as_millis();

    }

    let duration2: Duration = start2.elapsed();

    println!("terminó de cocinarse en {} segundos", duration2.as_secs());
    println!("tomó {} milisegundos en hacer la subimagen", tiempo_subimagen);
    println!("tomó {} milisegundos en hacer la comparacion", tiempo_comparar_imagenes);
    println!("tomó {} milisegundos en hacer el overlay", tiempo_overlay);
    
    return Ok(img_final)
}

fn imagen_mas_cercana_exacto(imagen_seccion: &RgbImage, images:& Vec<RgbImage>) -> RgbImage {
    // Función que encuentra la imagen que más se parece a una imagen de referencia 

    // Use a Mutex to protect the most_similar_image variable
    let most_similar_image: Mutex<image::ImageBuffer<image::Rgb<u8>, Vec<u8>>> = Mutex::new(image::ImageBuffer::new(0,0));
    let most_similarity: Mutex<f64> = Mutex::new(0.0);

        // Process each chunk in a separate thread
    images.into_par_iter().for_each(|image| {
        let similarity = compare_images(&imagen_seccion, &image);

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
    // Verificamos que ambas imágenes tengan el mismo tamaño
    assert_eq!(image1.dimensions(), image2.dimensions());

    let mut num_pixels_equal = 0;

    // Recorremos ambas imágenes pixel por pixel y comparamos los valores de cada uno
    for (x, y, pixel1) in image1.enumerate_pixels() {
        let pixel2 = image2.get_pixel(x, y);

        if color_close_enough(pixel1, pixel2, 20.0) {
            num_pixels_equal += 1;
        }
    }

    // Calculamos el porcentaje de parecido entre las imágenes
    let num_pixels = (image1.width() * image1.height()) as f64;
    let similarity = num_pixels_equal as f64 / num_pixels;

    return similarity
}

fn color_close_enough(rgb1: &image::Rgb<u8>, rgb2: &image::Rgb<u8>, closeness: f64) -> bool {
  
    let red_diff = (rgb1.0[0] as f32 - rgb2.0[0] as f32).abs() * 0.299;
    let green_diff = (rgb1.0[1] as f32 - rgb2.0[1] as f32).abs() * 0.587;
    let blue_diff = (rgb1.0[2] as f32 - rgb2.0[2] as f32).abs() * 0.114;

    let distance = (red_diff + green_diff + blue_diff) as f64;

    return distance < closeness 
}

// fn color_close_enough_gpu(rgb1: &GpuMat<Vec3b>, rgb2: &GpuMat<Vec3b>, closeness: f64) -> GpuMat<u8> {
//     let mut result = GpuMat::default();
// 
//     // Create a scalar value with the desired closeness threshold
//     let threshold = Scalar::new(closeness, 0.0, 0.0, 0.0);
// 
//     // Calculate the absolute difference between the two images
//     cuda::absdiff(rgb1, rgb2, &mut result);
// 
//     // Sum the values of all channels of the difference image
//     cuda::sum(result, &mut result);
// 
//     // Threshold the result to get a binary image indicating which pixels are close enough
//     cuda::threshold(&result, &mut result, threshold, 1.0, cv::THRESH_BINARY);
// 
//     return result;
// }
