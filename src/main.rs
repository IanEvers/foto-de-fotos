extern crate image;

use std::fs::{self};
use std::time::{Instant};
use image::{ImageBuffer,  RgbImage, DynamicImage, ImageError};
use palette::{ IntoColor, Lab, Pixel, Srgb,  FromColor};
use kmeans_colors::{get_kmeans, Kmeans, Sort};

fn main() {
    // borrar unwrap
    let paths = fs::read_dir("C:/Users/Ian/Desktop/imagenes/bathroom/")
    .unwrap()
    .filter_map(|e| e.ok())
    .map(|e| e.path().to_string_lossy().into_owned())
    .collect::<Vec<_>>();

    let dominante_imagenes = color_dominante_imagenes(paths);
    // println!("{:?}", dominante_imagenes)
}

fn color_dominante_imagenes(imagenes: Vec<String>) -> Vec<Lab> {

    println!("Arranco a analizar imágenes.");

    let start = Instant::now();

    let mut dominante_imagenes: Vec<Lab> = Vec::new();

    for (index, imagen) in imagenes.iter().enumerate() {
        dominante_imagenes.push(color_dominante_imagen(imagen));
        println!("{}/{}", index, imagenes.len());
    }
    
    let duration = start.elapsed();

    println!("Tardó {} segundos en analizar las imágenes recolectadas", duration.as_secs());

    return dominante_imagenes;
}

fn color_dominante_imagen(imagen: &String) -> Lab {
    
    let im = image::open(&imagen).unwrap().to_rgb8();

    let rgb: Vec<u8> = im.into_raw();

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


fn armar_imagen_objetivo(imagen: &str, dominante_imagenes:Vec<Lab>) ->Result<DynamicImage, ImageError>  {
    let filas : u32 = 100;
    let columnas : u32 = 100;
    
    
    let image = image::open(imagen)?;

    image.resize(4000, 4000, image::imageops::Lanczos3) ;
    Ok(image)


}