use nannou::prelude::*;
use chrono::{DateTime,Local};
use std::fs;

mod bsc;
use bsc::*;
use bsc::generate_bsc::create_simple_bsf;
use bsc::generate_shirataki::convert_sentence_2_shirataki;
use bsc::read_bcf::read_bcf;

//mujica setting
const TURNFACTOR : f32 = 0.001;
const SAFE_RANGE : f32 = 0.95;
const SPEED_FACTOR : f32 = 0.25;

//generate setting 
const SEQ : u32 = 14;
const IND1 : u32 = 6;
const IND2 : u32 = 8;

//display settings
const RESOLUTION : u32 = 70;
const SCALING : u32 = 500;
const COMPRESS_X : f32 = 1.;
const BOLDNESS : f32 = 5.;
//line alpha value
const ALPHA : u8 = 255;

//output_settings
const FRAME_LEN : u32 = 200;
const FRAME_RATE : u32 = 30;
const SAVE : bool = false;
const OUTPUT_DISCRIPTION : &str = "random_test";

//nannou Model
struct Model{
    bsc : BsCharactor,
    runtime : DateTime<Local>,
}

fn main(){
    nannou::app(model)
        .update(update)
        .run();
}

fn model(app: &App) -> Model{
    app.new_window().size(SCALING,SCALING).view(view).build().unwrap();

    let bsc;

    let path = "./assets/bcf/f000d10002.bcf";

    match read_bcf(path){
        None => {
            let pinfo = Pinfo { seq_num: SEQ, ind_num_v: vec![IND1,IND2] };
            bsc = create_simple_bsf(pinfo, 0.01)
        }
        Some(unw) =>  {
            bsc = unw
        }
    }

    let runtime : DateTime<Local>  = Local::now();

    Model { 
        bsc,
        runtime
    }
}

fn update(app: &App, model: &mut Model, _update: Update){
    let frame_interval : u64 = 60/FRAME_RATE as u64;

    //adjust update span
    if app.elapsed_frames() % frame_interval != 0 {
        return;
    }

    mujica(&mut model.bsc,TURNFACTOR,SAFE_RANGE,SPEED_FACTOR)
}

fn view(app: &App, model: &Model, frame: Frame) {

    let frame_interval : u64 = 60/FRAME_RATE as u64;

    //adjust fps
    if app.elapsed_frames() % frame_interval != 0 {
        return;
    }
    //seqential number
    let now_frame : u64 = app.elapsed_frames() / frame_interval;
    
    // Prepare to draw.
    let draw = app.draw();

    // Clear the background to dimgray
    frame.clear(WHITE);

    let shirataki : Shirataki = {

        let placebsc = PlaceBsc{place : vec2(0.,-0.),bsc : model.bsc.clone()};

        let sentence : Sentence = vec![placebsc];

        convert_sentence_2_shirataki(sentence, RESOLUTION, BOLDNESS, COMPRESS_X)

    };

    for j in shirataki.clone() {
        let iterator_points = for_pointes_colored(&j,SCALING as f32).into_iter();

        draw.path()
            .stroke()
            .end_cap_round()
            .weight(1.)
            .points_colored(iterator_points);

    }

    //output seqential png
    let seq_num_string = {
        let i : u64 = now_frame;

        if i < 10 {format!("000{}",i)}
        else if i < 100 {format!("00{}",i)}
        else if i < 1000 {format!("0{}",i)}
        else {format!("{}",i)}
    };

    if SAVE == true {
        let time_string :String  = model.runtime.format("%Y_%m_%d_%H_%M_%S").to_string(); 

        if now_frame <= 0 {
            fs::create_dir(format!("./assets/img/{}_{}", time_string, OUTPUT_DISCRIPTION,))
                .expect("Failed to create directory");
        }

        let file_name : String = format!("./assets/img/{}_{}/{}_{}_{}.png", time_string, OUTPUT_DISCRIPTION, time_string, OUTPUT_DISCRIPTION, seq_num_string);

        if now_frame < FRAME_LEN as u64{
            app.main_window().capture_frame(file_name);
            println!("{}",seq_num_string);
        }

    }

    draw.to_frame(app, &frame).unwrap();
}

//convert d2line so that nannou can draw line
fn for_pointes_colored(input : &D2Line, scaling : f32) ->  Vec<(Vec2,Rgba8)> {
    let mut out_put = Vec::new();
    for i in input {

        let x_nannou = (i.x - 0.5) * scaling;
        let y_nannou = (i.y - 0.5) * scaling;
        out_put.push((vec2(x_nannou,y_nannou),rgba8(240,180,170,ALPHA)));
    }
    out_put
}

fn mujica(bsc : &mut BsCharactor,turnfactor : f32,safe_range : f32,speed_factor : f32){
    
    for i in 0..bsc.seq.len(){
        bsc.seq[i] = move_point(bsc.seq[i],turnfactor,safe_range,speed_factor)
    }

    for i in 0..bsc.ind.len(){
        for j in 0..bsc.ind[i].len(){
            bsc.ind[i][j] = move_point(bsc.ind[i][j],turnfactor,safe_range,speed_factor)
        }
    }
}

fn move_point(v4 : Vec4,turnfactor : f32,safe_range : f32,speed_factor : f32) -> Vec4{
    let x = v4.x;
    let y = v4.y;
    let mut vx = v4.z;
    let mut vy = v4.w;

    if x - 0.5 < -0.5 * safe_range{
        vx += turnfactor * speed_factor;
    }
    if 0.5 * safe_range < x - 0.5 {
        vx -= turnfactor * speed_factor;
    }
    if y - 0.5 < -0.5 * safe_range{
        vy += turnfactor * speed_factor;
    }
    if 0.5 * safe_range < y - 0.5 {
        vy -= turnfactor * speed_factor;
    }

    vec4(x+vx*speed_factor,y+vy*speed_factor,vx,vy)
}