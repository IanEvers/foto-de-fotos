extern crate image;
extern crate rayon;
extern crate rand;

use std::fs::{self};
use rand::Rng;
use std::path::{Path, self};
use std::thread;
use std::time::{Instant};
use image::imageops::{resize, FilterType};
use image::{RgbImage, DynamicImage, ImageError, SubImage, open};
use palette::{IntoColor, Lab,  Srgb, Pixel as OtherPixel};
use palette::rgb::Rgb;
use kmeans_colors::{get_kmeans, Kmeans, Sort};
use deltae::*;
use rayon::prelude::*;

struct ImagenLab {
    ubicacion: String,
    lab: Rgb
}

fn main() {
    // let paths = fs::read_dir("C:/Users/Ian/Desktop/imagenes/beautiful landscape/")
    // .unwrap()
    // .filter_map(|e| e.ok())
    // .map(|e| e.path().to_string_lossy().into_owned())
    // .collect::<Vec<_>>();

    let imagen_objetivo = "C:/Users/Ian/Pictures/dalle.png";

    // let dominante_imagenes = color_dominante_imagenes(paths);

    let imagen_final_result = armar_imagen_objetivo(imagen_objetivo);
    
    let mut rng = rand::thread_rng();

    let random_value = rng.gen_range(0..100).to_string();
    
    let image_name = "imagenPrueba".to_owned() + &random_value + ".png";

    match imagen_final_result {
        Ok(imagen) => imagen.save(image_name).unwrap(),
        Err(_) => println!("hubo error."),
    };
}

fn load_images_from_dir(dir: &Path) -> Vec<RgbImage> {

    // Obtenemos todos los archivos en el directorio
    // let entries = fs::read_dir(dir).unwrap();
   
    // Recorremos todos los archivos y cargamos las imágenes que encontremos

    
    let paths = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    let images = paths.par_iter().map(|path| load_image(path)).collect();


    // for entry in entries {
    //     let entry = entry.unwrap();
    //     println!("{}", entry.path().to_string_lossy());

        
    //     // Verificamos si el archivo es una imagen
    //     if let Some(ext) = entry.path().extension() {
    //         if ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "bmp" || ext == "gif" {
    //             let start_abrir = Instant::now();
    //             // Cargamos la imagen
    //             let image: RgbImage = image::open(entry.path()).unwrap().to_rgb8();
    //             let duracion_abrir = start_abrir.elapsed();
    //             tiempo_abrir += duracion_abrir.as_millis();
    //             println!("{}", tiempo_abrir);
    //             images.push(image);
    //         }
    //     }
    // }
    return images
}

fn load_image(image_path: &str) -> RgbImage {
    println!("{}", image_path);
    return image::open(image_path).unwrap().to_rgb8();
}
    
fn color_dominante_imagenes(imagenes: Vec<String>) -> Vec<ImagenLab> {

    println!("Arranco a analizar imágenes.");

    let start = Instant::now();

    let mut dominante_imagenes: Vec<ImagenLab> = Vec::new();

    let mut tiempo_abrir_imagen: u128 = 0;  
    let mut tiempo_analizar_imagen: u128 = 0; 

    for (index, imagen) in imagenes.iter().enumerate() {
        println!("{}/{} {}", index, imagenes.len(), imagenes[index]);
        
        let start_abrir = Instant::now();
        let im: RgbImage = image::open(&imagen).unwrap().to_rgb8();
        let duracion_abrir = start_abrir.elapsed();
        tiempo_abrir_imagen += duracion_abrir.as_millis();

        let start_analizar = Instant::now();

        let im_width= im.width();
        let im_height= im.height();

        dominante_imagenes.push({
            ImagenLab {
                ubicacion: imagen.to_string(),
                lab: color_promedio(&im, 0, im_width, 0, im_height)
            }
        });

        let duracion_analizar = start_analizar.elapsed();
        tiempo_analizar_imagen += duracion_analizar.as_millis();

    }

    println!("{} milisegundos en analizar", tiempo_analizar_imagen);
    println!("{} milisegundos en abrir", tiempo_abrir_imagen);
    
    let duration = start.elapsed();

    println!("Tardó {} segundos en analizar las imágenes recolectadas", duration.as_secs());

    return dominante_imagenes;
}

fn color_dominante_imagen(imagen: RgbImage) -> Lab {
    
    let rgb: Vec<u8> = imagen.into_raw();

    let lab: Vec<Lab> = Srgb::from_raw_slice(&rgb)
    .iter()
    .map(|x| x.into_format().into_color())
    .collect();

    // Iterate over the runs, keep the best results
    let mut result = Kmeans::new();
    for _i in 0..2 {
        let run_result = get_kmeans(
            2,
            3,
            0.9,
            false,
            &lab,
            20000,
        );
        if run_result.score < result.score {
            result = run_result;
        }
    }

    // Using the results from the previous example, process the centroid data
    let res = Lab::sort_indexed_colors(&result.centroids, &result.indices);

    // We can find the dominant color directly
    let dominant_color = Lab::get_dominant_color(&res);

    // borrar unwrap
    match dominant_color {
        Some(color_lab) => return color_lab,
        None => println!("Fallo obtener color dominante"),
    };
    //En caso de no encontrar color dominante se devuelve el 0
    Lab::new(0.0, 0.0, 0.0)
}

fn color_promedio(imagen: &RgbImage, principio_x: u32, final_x: u32, principio_y: u32, final_y: u32) -> Rgb {
    
    let mut r_total: u32 = 0;
    let mut g_total: u32 = 0;
    let mut b_total: u32 = 0;

    let area_seccion = (final_x - principio_x) * (final_y - principio_y);

    for x in principio_x..final_x {
        for y in principio_y..final_y {

            let rgb = imagen.get_pixel(x, y).0;

            r_total += rgb[0] as u32;
            g_total += rgb[1] as u32;
            b_total += rgb[2] as u32;
        }
    }

    let r_promedio = (r_total / area_seccion) as f32;
    let g_promedio = (g_total / area_seccion) as f32;
    let b_promedio = (b_total / area_seccion) as f32;

    let color_promedio_rgb: Rgb = Rgb::new(r_promedio.into() , g_promedio.into(), b_promedio.into());

    // let color_promedio_lab: Lab = color_promedio_rgb.into_color();
    
    return color_promedio_rgb

}

fn armar_imagen_objetivo(imagen: &str) -> Result<RgbImage, ImageError> {

    println!("preparo.");

    let start_preparo = Instant::now();
    let filas : u32 = 100;
    let columnas : u32 = 100;
    
    let image = image::open(imagen).unwrap().to_rgb8();

    let ancho = 4000;
    let alto= 4000;

    let image_rgb_data = load_images_from_dir(Path::new("C:/Users/Ian/Desktop/imagenes/unsplash_resized"));

    let mut image_rgb_data_resized = Vec::new();

    for image in image_rgb_data {
        let resized_image = image::imageops::resize(&image, ancho / columnas, alto / filas, FilterType::Nearest);
        image_rgb_data_resized.push(resized_image);
    }

    let image_resized = image::imageops::resize(&image, ancho, alto, image::imageops::Nearest);

    let ancho_seccion: u32 = ancho / columnas;
    let alto_seccion: u32 = alto / filas;

    let mut img_final: RgbImage = image::ImageBuffer::new(ancho, alto);

    let mut principio_seccion_x: u32;
    let mut principio_seccion_y: u32;

    let duration = start_preparo.elapsed();

    println!("tarde {} en preparar.", duration.as_secs());

    println!("empiezo.");

    let start2 = Instant::now();

    for seccion in 0..(filas * columnas) {

        println!("{} / {}", seccion, filas * columnas);

        if seccion > 0 {
            principio_seccion_x = (seccion * ancho_seccion) % ancho;
            principio_seccion_y = (seccion / filas) * alto_seccion;
        } else {
            principio_seccion_x = 0;
            principio_seccion_y = 0;
        }

        let subimagen = SubImage::new(&image_resized, principio_seccion_x, principio_seccion_y, ancho_seccion, alto_seccion).to_image();
        
        // let subimagen_dynamic: DynamicImage = DynamicImage::ImageRgba8(subimagen);

        // let color_dominante_seccion = color_dominante_imagen(subimagen_dynamic.to_rgb8());
        

        // let color_promedio_seccion = color_promedio(&image, principio_seccion_x, principio_seccion_x + ancho_seccion, principio_seccion_y, principio_seccion_y + alto_seccion);

        // let imagen_seccion = imagen_mas_cercana(color_promedio_seccion, &image_rgb_data);
        
        let imagen_seccion: RgbImage = imagen_mas_cercana_exacto(&subimagen, &image_rgb_data_resized);

        // let imagen_seccion_archivo: DynamicImage = image::open(&imagen_seccion.ubicacion).unwrap();

        // let imagen_seccion_archivo_resized: RgbImage = imagen_seccion_archivo.resize_exact(ancho_seccion, alto_seccion, image::imageops::Nearest).to_rgb8();
        
        image::imageops::overlay(&mut img_final, &imagen_seccion, principio_seccion_x.into(), principio_seccion_y.into());
    }

    let duration2 = start2.elapsed();

    println!("terminó de cocinarse en {} segundos", duration2.as_secs());

    return Ok(img_final)
}

fn imagen_mas_cercana(color_dominante_seccion: Rgb, dominante_imagenes: &Vec<ImagenLab>) -> ImagenLab {
    let mut distancia_mas_cercana: f32 = 10000.0;
    let mut imagen_mas_cerca: ImagenLab = ImagenLab { ubicacion: "null".to_string(), lab: Rgb::new(0.0, 0.0, 0.0) }; 
    // let mut imagen_mas_cerca: ImagenLab = ImagenLab { ubicacion: "null".to_string(), lab: Lab { l: 1.0, a: 1.0, b: 1.0, white_point: PhantomData } }; 

    for i in 0..dominante_imagenes.len() {
        let distancia: f32 = distancia_entre_dos_colores_rgb(color_dominante_seccion, dominante_imagenes[i].lab);
        if distancia_mas_cercana == 10000.0 || distancia < distancia_mas_cercana {
            distancia_mas_cercana = distancia;
            imagen_mas_cerca = ImagenLab {
                ubicacion: dominante_imagenes[i].ubicacion.to_string(),
                lab: dominante_imagenes[i].lab
            };
        }
    }

    return imagen_mas_cerca
}

fn distancia_entre_dos_colores_lab(lab1: Lab, lab2: Lab) -> f32 {
    
    let lab1 = LabValue {
        l: lab1.l,
        a: lab1.a,
        b: lab1.b,
    };

    let lab2 = LabValue {
        l: lab2.l,
        a: lab2.a,
        b: lab2.b,
    };

    return *DeltaE::new(&lab1, &lab2, DE2000).value();
}

fn distancia_entre_dos_colores_rgb(rgb1: Rgb, rgb2: Rgb) -> f32 {
    return (rgb1.red - rgb2.red) * (rgb1.red - rgb2.red) + (rgb1.green - rgb2.green) * (rgb1.green - rgb2.green) + (rgb1.blue - rgb2.blue) * (rgb1.blue - rgb2.blue)
}

fn imagen_mas_cercana_exacto(imagen_objetivo: &RgbImage, images:& Vec<RgbImage>) -> RgbImage {
    // Función que encuentra la imagen que más se parece a una imagen de referencia 
    let mut most_similar_image = image::ImageBuffer::new(0,0);
    let mut most_similarity = 0.0;

    for image in images {
        let similarity = compare_images(&imagen_objetivo, &image);

        if similarity > most_similarity {
            most_similar_image = image.clone();
            most_similarity = similarity;
        }
    }

    return most_similar_image
}

// Función que compara dos imágenes pixel por pixel y devuelve su parecido en términos de porcentaje
fn compare_images(image1: &RgbImage, image2: &RgbImage) -> f64 {
    // Verificamos que ambas imágenes tengan el mismo tamaño
    assert_eq!(image1.dimensions(), image2.dimensions());

    let mut num_pixels_equal = 0;

    // Recorremos ambas imágenes pixel por pixel y comparamos los valores de cada uno
    for (x, y, pixel1) in image1.enumerate_pixels() {
        let pixel2 = image2.get_pixel(x, y);

        if color_close_enough(pixel1, pixel2, 25.0) {
            num_pixels_equal += 1;
        }
    }

    // Calculamos el porcentaje de parecido entre las imágenes
    let num_pixels = (image1.width() * image1.height()) as f64;
    let similarity = (num_pixels_equal as f64 / num_pixels) * 50.0;

    return similarity
}

fn color_close_enough(rgb1: &image::Rgb<u8>, rgb2: &image::Rgb<u8>, closeness: f64) -> bool {
  
    let red_diff = (rgb1.0[0] as i32 - rgb2.0[0] as i32).abs();
    let green_diff = (rgb1.0[1] as i32 - rgb2.0[1] as i32).abs();
    let blue_diff = (rgb1.0[2] as i32 - rgb2.0[2] as i32).abs();

    let distance = (red_diff + green_diff + blue_diff) as f64;

    if distance < closeness {
        return true;
    } else {
        return false;
    }
}