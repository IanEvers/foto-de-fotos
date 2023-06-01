mod vec2d;

extern crate image;
extern crate rayon;
extern crate rand;

use std::fs::{self};
use rand::Rng;
use tqdm::Iter;
use std::path::{Path};
use std::time::{Instant, Duration};
use image::imageops::{FilterType};
use image::{RgbImage, ImageError, SubImage, ImageBuffer};
use rayon::{prelude::*};
use std::sync::Mutex;
use vec2d::Vec2d;

#[derive(Clone)]
struct ImagenSeccion {
    index: usize,
    similarity: f64
}

struct ImagenCarpeta {
    index: usize,
    image: RgbImage
}


fn main() {
    
    let imagen_objetivo = "./adriel.jpg";

    let imagen_final_result = armar_imagen_objetivo(imagen_objetivo, "D:/movie posters/", 5);
    
    let random_value = rand::thread_rng().gen_range(0..500).to_string();
    
    let image_name = "imagenPrueba".to_owned() + &random_value + ".png";
    println!("{}", image_name);

    match imagen_final_result {
        Ok(imagen) => imagen.save(image_name).unwrap(),
        Err(_) => println!("hubo error."),
    };

    println!("listo");
}

fn load_images_from_dir(dir: &Path, from: i16, to: i16) -> Vec<ImagenCarpeta> {

    // Recorremos todos los archivos y cargamos las imágenes que encontremos
    let paths = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    let images: Vec<ImagenCarpeta> = paths.par_iter()
    .enumerate()
    .filter(|(i, _)| *i >= from as usize && *i <= to as usize)
    .map(|(index, path)| {
         match image::open(path) {
            Ok(img) => ImagenCarpeta {
                image: img.to_rgb8(),
                index: index 
            },
            Err(_) => ImagenCarpeta {
                image: ImageBuffer::new(1, 1),
                index: index 
            },
            
        }
    })
    .collect();

    return images
}

fn armar_imagen_objetivo(imagen: &str, images_path: &str, rango_imagenes: i16) -> Result<RgbImage, ImageError> {

    println!("preparo.");

    let image: RgbImage = image::open(imagen).unwrap().to_rgb8();

    let ancho: u32 = 2100;
    let alto: u32 = 2100;
    
    let columnas: u32 = 30;
    let filas: u32 = 30;

    assert!(ancho % columnas == 0, "el ancho tiene que ser divisible por la cantidad de columnas");
    assert!(alto % filas == 0, "el alto tiene que ser divisible por la cantidad de filas");

    let ancho_seccion: u32 = ancho / columnas;
    let alto_seccion: u32 = alto / filas;

    let mut img_final: RgbImage = image::ImageBuffer::new(ancho, alto);

    let mut principio_seccion_x: u32;
    let mut principio_seccion_y: u32;

    println!("empiezo.");

    let start2 = Instant::now();

    let mut tiempo_cuentas = 0;
    let mut tiempo_resize = 0;
    let mut tiempo_subimagen = 0;
    let mut tiempo_load_total = 0;

    let mut lista_imagenes_mas_similares: Vec<ImagenSeccion> = vec![ImagenSeccion { index: 1, similarity: 0.0 }; (filas * columnas) as usize];
    
    
    for (chunk_start, chunk_end) in split_directory_chunks(images_path, 10000) {

        let start_load_chunk = Instant::now();
        let mut image_rgb_data: Vec<ImagenCarpeta> = load_images_from_dir(Path::new(images_path), chunk_start, chunk_end);
        let tiempo_load_chunk = start_load_chunk.elapsed().as_secs();
        println!("este chunk de {} a {} tardo {} segundos", chunk_start, chunk_end, tiempo_load_chunk);
        tiempo_load_total += tiempo_load_chunk;
        
        for seccion in (0..(filas * columnas)).into_iter().tqdm() {
            
            if seccion > 0 {
                principio_seccion_x = (seccion * ancho_seccion) % ancho;
                principio_seccion_y = (seccion / columnas) * alto_seccion;
            } else {
                principio_seccion_x = 0;
                principio_seccion_y = 0;
            }

            let principio_seccion_x_base: f32 = (principio_seccion_x as f32 / img_final.width() as f32) * image.width() as f32;
            let principio_seccion_y_base: f32 = (principio_seccion_y as f32 / img_final.height() as f32) * image.height() as f32;
            
            let ancho_seccion_base: f32 = (ancho_seccion as f32 / img_final.width() as f32) * image.width() as f32;
            let alto_seccion_base: f32 = (alto_seccion as f32 / img_final.height() as f32) * image.height() as f32;
            
            let start_subimagen = Instant::now();
            let subimagen: RgbImage = SubImage::new(&image, principio_seccion_x_base as u32, principio_seccion_y_base as u32, ancho_seccion_base as u32, alto_seccion_base as u32).to_image();
            tiempo_subimagen += start_subimagen.elapsed().as_millis();

            let start_cuentas = Instant::now();
            let imagen_mas_similar: ImagenSeccion = imagen_mas_similar(&subimagen, &mut image_rgb_data, lista_imagenes_mas_similares.clone(), rango_imagenes, seccion as i16, filas as i16, columnas as i16);

            match lista_imagenes_mas_similares.get(seccion as usize) {
                Some(_) => {
                    if imagen_mas_similar.similarity > lista_imagenes_mas_similares[seccion as usize].similarity  {
                        lista_imagenes_mas_similares[seccion as usize] = imagen_mas_similar;
                    }
                },
                None => lista_imagenes_mas_similares.push(imagen_mas_similar)
            }

            tiempo_cuentas += start_cuentas.elapsed().as_millis();
        }
    }

    println!("Ya tengo los valores, empiezo a crear la imagen");

    for seccion in (0..(filas * columnas)).into_iter().tqdm() {

        if seccion > 0 {
            principio_seccion_x = (seccion * ancho_seccion) % ancho;
            principio_seccion_y = (seccion / columnas) * alto_seccion;
        } else {
            principio_seccion_x = 0;
            principio_seccion_y = 0;
        }
        
        let index_imagen_seccion: usize = lista_imagenes_mas_similares[seccion as usize].index;
        
        let files = fs::read_dir(images_path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();

        let start_resize = Instant::now();
        let resized_image_seccion: RgbImage = image::imageops::resize(&image::open(&files[index_imagen_seccion]).unwrap().to_rgb8(), ancho / columnas, alto / filas, FilterType::Nearest);
        tiempo_resize += start_resize.elapsed().as_millis();
        
        image::imageops::overlay(&mut img_final, &resized_image_seccion, principio_seccion_x.into(), principio_seccion_y.into());
    }

    let duration2: Duration = start2.elapsed();

    println!("terminó de cocinarse en {} segundos", duration2.as_secs());
    println!("tiempo comparando: {} milisegundos", tiempo_cuentas);
    println!("tiempo resizeando: {} milisegundos", tiempo_resize);
    println!("tiempo subimagen: {} milisegundos", tiempo_subimagen);
    println!("tiempo load_chunk: {} milisegundos", tiempo_load_total);
  
    return Ok(img_final)
}

fn split_directory_chunks(path: &str, max_size: i16) -> Vec<(i16, i16)> {

    let files = fs::read_dir(path)
    .unwrap()
    .map(|entry| entry.unwrap().path())
    .collect::<Vec<_>>();

    // Determine the number of chunks based on the max size
    let num_chunks = (files.len() as f32 / max_size as f32).ceil() as i16;

    // Generate the chunks
    return (0..num_chunks)
        .map(|i| {
            let start = i * max_size;
            let end = (i + 1) * max_size - 1;
            let end = std::cmp::min(end, files.len() as i16 - 1);
            return (start, end)
        })
        .collect()
}

// Función que encuentra la imagen que más se parece a una imagen de referencia 
fn imagen_mas_similar(imagen_seccion: &RgbImage, images: &mut Vec<ImagenCarpeta>, lista_imagenes_mas_similares:  Vec<ImagenSeccion>, rango_imagenes: i16, current_index: i16, filas: i16, columnas: i16) -> ImagenSeccion {

    let imagen_mas_similar = Mutex::new(0);
    let most_similarity: Mutex<f64> = Mutex::new(0.0);
    
    // Process each chunk in a separate thread
    images.par_iter().for_each(|imagen_carpeta| {
       
        let similarity = compare_images(&imagen_seccion, &imagen_carpeta.image, 100);
        
        if similarity > *most_similarity.lock().unwrap() {
            if !image_in_range(imagen_carpeta.index, current_index, lista_imagenes_mas_similares.clone(), rango_imagenes, filas, columnas) {
                *imagen_mas_similar.lock().unwrap() = imagen_carpeta.index;
                *most_similarity.lock().unwrap() = similarity;
            }
        }
    });
    
    return ImagenSeccion {
        similarity: most_similarity.into_inner().unwrap(),
        index: imagen_mas_similar.into_inner().unwrap()
    }
}

fn image_in_range(image_folder_index: usize, current_index: i16, lista_imagenes_mas_similares: Vec<ImagenSeccion>, rango_imagenes: i16, filas: i16, columnas: i16) -> bool {
    // recorro ciertas posiciones de lista_imagenes_mas_similares relativas a current_index,
    // si en alguna de esas posiciones existe una imagen cuyo index sea igual a image_folder_index, retorno true.

    let columna_actual = current_index % filas;
    let fila_actual = (current_index - columna_actual) / filas;
    let matriz = Vec2d::new(lista_imagenes_mas_similares, filas as usize, columnas as usize);

    let fila_inicial = if fila_actual - rango_imagenes < 0 {0} else {fila_actual - rango_imagenes};
    let columna_inicial = if columna_actual - rango_imagenes < 0 {0} else {columna_actual - rango_imagenes};

    // println!("accediendo a  current index: {} ", current_index);

    for x in (fila_inicial)..=(fila_actual) {
        for y in (columna_inicial)..=(columna_actual) {
            // println!("accediendo a la matriz, valores fila:{} columna:{}, current index: {} ",x,y, current_index);
            if matriz.index(x as usize, y as usize).index == image_folder_index {
                return true;
            }
        }
    }
    
    false
}

// Función que compara dos imágenes pixel por pixel y devuelve su parecido en términos de porcentaje
fn compare_images(image1: &RgbImage, image2: &RgbImage, presicion: i32) -> f64 {
    let mut num_pixels_equal = 0;
    // Recorremos ambas imágenes pixel por pixel y comparamos los valores de cada uno
    for (x, y, pixel1) in image1.enumerate_pixels().step_by((100 / presicion) as usize) {
        let pixel_x: f32 = ((x as f32 / image1.width() as f32) * image2.width() as f32).into();
        let pixel_y: f32 = ((y as f32 / image1.height() as f32) * image2.height() as f32).into();
 
        let pixel2 = image2.get_pixel(pixel_x as u32, pixel_y as u32);

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