use {
    image::{
        codecs::gif::{
            GifDecoder,
            GifEncoder,
        },
        io::Reader as ImageReader,
        AnimationDecoder,
        Frame,
        Rgba,
        RgbaImage,
    },
    itertools::Itertools,
    std::{
        fs::File,
        io::{
            self,
            Write,
        },
        path::PathBuf,
        thread,
    },
};

fn process(frame: &mut Frame, mask: &RgbaImage) {
    let frame_buffer = frame.buffer_mut();

    for x in 0..frame_buffer.width() {
        for y in 0..frame_buffer.height() {
            let a = frame_buffer[(x, y)].0;
            let b = mask[(x, y)].0;

            let mul = |i| (a[i] as u16 * b[i] as u16 / u8::MAX as u16) as u8;
            let ab = [mul(0), mul(1), mul(2), mul(3)];
            let ab = Rgba::from(ab);

            frame_buffer[(x, y)] = ab;
        }
    }
}

fn print_flush(prompt: &str) {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
}

fn read_path() -> PathBuf {
    PathBuf::from(io::stdin().lines().next().unwrap().unwrap())
}

fn main() {
    let gif_path = {
        print_flush("путь к гифке: ");
        read_path()
    };
    assert!(gif_path.exists(), "гифки по указанному пути не существует");

    let mask_path = {
        print_flush("путь к маске: ");
        read_path()
    };
    assert!(gif_path.exists(), "маски по указанному пути не существует");

    let output_path = {
        print_flush("выходной путь: ");
        read_path()
    };
    let output_file = File::create(output_path).expect("не получилось создать выходной файл");

    let mask = image::open(mask_path).unwrap().into_rgba8();

    let gif = ImageReader::open(gif_path).unwrap();

    println!("загружаю гифку...");
    let mut frames = GifDecoder::new(gif.into_inner())
        .unwrap()
        .into_frames()
        .map(|frame| frame.unwrap())
        .collect_vec();
    println!("гифка загружена");

    assert_eq!(
        mask.width(),
        frames[0].buffer().width(),
        "ширина маски не совпадает с шириной гифки"
    );
    assert_eq!(
        mask.height(),
        frames[0].buffer().height(),
        "высота маски не совпадает с шириной гифки"
    );

    thread::scope(|scope| {
        let mask = &mask;

        frames
            .iter_mut()
            .map(|frame| scope.spawn(move || process(frame, mask)))
            .collect_vec()
            .into_iter()
            .for_each(|thread| thread.join().unwrap());
    });

    println!("умножил, сохраняю...");
    GifEncoder::new_with_speed(output_file, 30)
        .encode_frames(frames)
        .unwrap();
    println!("готово");
}
