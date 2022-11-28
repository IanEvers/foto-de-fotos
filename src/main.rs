extern crate image;

use std::env;
use std::fmt::DebugSet;
use std::fs::{self};
use std::marker::PhantomData;
use std::time::{Instant};
use image::{RgbImage, DynamicImage, ImageError, SubImage};
use palette::{IntoColor, Lab, Pixel, Srgb};
use palette::rgb::Rgb;
use kmeans_colors::{get_kmeans, Kmeans, Sort};
use deltae::*;

struct ImagenLab {
    ubicacion: String,
    lab: Lab
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let paths = fs::read_dir("C:/Users/Ian/Desktop/imagenes/bathroom/")
    .unwrap()
    .filter_map(|e| e.ok())
    .map(|e| e.path().to_string_lossy().into_owned())
    .collect::<Vec<_>>();

    let imagen_objetivo = "C:/Users/Ian/Pictures/dalle.png";

    let dominante_imagenes = color_dominante_imagenes(paths);

    let imagen_final_result = armar_imagen_objetivo(imagen_objetivo, dominante_imagenes);

    match imagen_final_result {
        Ok(imagen) => imagen.save("imagenPrueba2.png").unwrap(),
        Err(_) => println!("hubo error."),
    };
}

fn color_dominante_imagenes(imagenes: Vec<String>) -> Vec<ImagenLab> {

    println!("Arranco a analizar imágenes.");

    let start = Instant::now();

    let mut dominante_imagenes: Vec<ImagenLab> = Vec::new();

    for (index, imagen) in imagenes.iter().enumerate() {
        let im: RgbImage = image::open(&imagen).unwrap().to_rgb8();
        dominante_imagenes.push({
            ImagenLab {
                ubicacion: imagen.to_string(),
                lab: color_dominante_imagen(im)
            }
        });
        println!("{}/{}", index, imagenes.len());
    }
    
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
    for i in 0..2 {
        let run_result = get_kmeans(
            3,
            2,
            0.5,
            false,
            &lab,
            i as u64,
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


fn armar_imagen_objetivo(imagen: &str, dominante_imagenes:Vec<ImagenLab>) -> Result<RgbImage, ImageError> {

    println!("preparo.");

    let start = Instant::now();

    let filas : u32 = 100;
    let columnas : u32 = 100;
    
    let image = image::open(imagen).unwrap();

    let ancho = 2000;
    let alto= 2000;

    let image_resized = image.resize_exact(ancho, alto, image::imageops::Lanczos3);

    let ancho_seccion: u32 = ancho / columnas;
    let alto_seccion: u32 = alto / filas;

    let mut img_final: RgbImage = image::ImageBuffer::new(ancho, alto);

    let mut principio_seccion_x: u32;
    let mut principio_seccion_y: u32;

    let duration = start.elapsed();

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

        let subimagen_dynamic: DynamicImage = DynamicImage::ImageRgba8(subimagen);

        let color_dominante_seccion = color_dominante_imagen(subimagen_dynamic.to_rgb8());

        let imagen_seccion = imagen_mas_cercana(color_dominante_seccion, &dominante_imagenes);

        let imagen_seccion_archivo: DynamicImage = image::open(&imagen_seccion.ubicacion).unwrap();

        let imagen_seccion_archivo_resized: RgbImage = imagen_seccion_archivo.resize_exact(ancho_seccion, alto_seccion, image::imageops::Nearest).to_rgb8();
        
        image::imageops::overlay(&mut img_final, &imagen_seccion_archivo_resized, principio_seccion_x.into(), principio_seccion_y.into());
        
        // let color: Rgb = color_dominante_seccion.into_color();
        // println!("{}" , (color.red * 255.0) as u8 );

        // for (x, y, pixel) in img_buf_prueba.enumerate_pixels_mut() {
            
        //     let r = (0.3 * x as f32) as u8;
        //     let b = (0.3 * y as f32) as u8;
        //     // *pixel = image::Rgb([r, 200, b]);

        //     *pixel = image::Rgb([(color.red * 255.0) as u8, (color.green * 255.0) as u8, (color.blue * 255.0) as u8]);
        // }

        // image::imageops::overlay(&mut img_final, &img_final, principio_seccion_x.into(), principio_seccion_y.into());
    }

    let duration2 = start2.elapsed();

    println!("terminó de cocinarse en {} segundos", duration2.as_secs());

    return Ok(img_final)
}

fn imagen_mas_cercana(color_dominante_seccion: Lab, dominante_imagenes: &Vec<ImagenLab>) -> ImagenLab {
    let mut distancia_mas_cercana: f32 = 10000.0;
    let mut imagen_mas_cerca: ImagenLab = ImagenLab { ubicacion: "null".to_string(), lab: Lab { l: 1.0, a: 1.0, b: 1.0, white_point: PhantomData } }; 

    for i in 0..dominante_imagenes.len() {
        let distancia: f32 = distancia_entre_dos_colores(color_dominante_seccion, dominante_imagenes[i].lab);
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

fn distancia_entre_dos_colores(lab1: Lab, lab2: Lab) -> f32 {
    
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
