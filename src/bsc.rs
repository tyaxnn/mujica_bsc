use nannou::geom::*;

//Charactor Component (from primitive to complicated)
    //one stroke
    pub type D4Line = Vec<Vec4>;
    //one charactor
    #[derive(Clone)]
    pub struct BsCharactor{
        pub seq : D4Line,
        pub ind : Vec<D4Line>,
    }
    //one sentence
    pub type Sentence = Vec<PlaceBsc>;

//define useful struct when generate new charactor
    pub struct Pinfo{
        pub seq_num : u32,
        pub ind_num_v : Vec<u32>,
    }

    pub struct PlaceBsc{
        pub place : Vec2,
        pub bsc : BsCharactor,
    }


//define useful type when output
    //one 2_dementional_line
    pub type D2Line = Vec<Vec2>;
    //an assembly of many lines
    pub type Shirataki = Vec<D2Line>;
    //an assembly of many 4d lines
    pub type D4shirataki = Vec<D4Line>;

pub mod generate_bsc {
    use rand::Rng;
    use crate::bsc::*;

    //completely randomly create bsc
    pub fn create_simple_bsf(pinfo : Pinfo,v_strength : f32) -> BsCharactor{

        let mut seq : D4Line = Vec::new();
        let mut ind : Vec<D4Line> = Vec::new();
        //create seq
        for _ in 0..pinfo.seq_num{
            seq.push(random_d4(v_strength));
        }

        if seq[seq.len()-1].x < seq[0].x{
            seq.reverse();
        }
        //create ind
        for ind_num in pinfo.ind_num_v{
            let mut a_ind : D4Line = Vec::new();
            for _ in 0..ind_num{
                a_ind.push(random_d4(v_strength));
            }

            ind.push(a_ind);
        }

        BsCharactor {
            seq,
            ind,
        }
    }

    //a function create random 2d vector
    fn random_vec2(max_vec2_norm : f32) -> Vec2{
        //let normal = Normal::new(0.0, max_vec2_norm).unwrap();

        let mut rng = rand::thread_rng();

        let x = (rng.gen::<f32>() - 0.5) * 2. * max_vec2_norm;
        let y = (rng.gen::<f32>() - 0.5) * 2. * max_vec2_norm;
        let rand_vec = vec2(x,y);

        rand_vec
    }

    //create random vec4
        //coordinate center (0.5,0.5),range[0,1]
        //verocity range[-vmax,vmax]
    fn random_d4(v_strength : f32)-> Vec4{
        let coordinate = random_vec2(0.5) + vec2(0.5,0.5);

        let verosity = random_vec2(v_strength);

        vec4(coordinate.x,coordinate.y,verosity.x,verosity.y)
    }    
}

pub mod generate_shirataki {
    use crate::bsc::*;
    use bspline;


    //convert bsc to shirataki ,which is useful for nannou
    fn _convert_bsc_2_shirataki(bsc : BsCharactor,resolution : u32, boldness : f32) -> Shirataki{
        let mut d4shirataki : D4shirataki = Vec::new();

        //bsc pack into 4 dementional shirataki
        d4shirataki.push(bsc.seq.clone());
        for a_ind in bsc.ind.clone(){
            d4shirataki.push(a_ind);
        }

        //make smooth by bspline
        for i in 0..d4shirataki.len(){
            d4shirataki[i] = bspline_for_d4line(&d4shirataki[i]);
        }

        //convert to shirataki
        let shirataki = d4_2_d2(d4shirataki,resolution,boldness);
        shirataki
    }

    //convert sentence to shirataki ,which is useful for nannou
    pub fn convert_sentence_2_shirataki(sentence : Sentence,resolution : u32, boldness : f32, compress_x : f32) -> Shirataki{
        let mut seq_part : D4Line = Vec::new();
        let mut d4shirataki : D4shirataki = Vec::new();

        let mut y_before = 0.;

        //loop for all bsc
        for one_p_bsc in sentence{
            //change coordinate by place
            let place : Vec2 = one_p_bsc.place;
            let seq : D4Line = one_p_bsc.bsc.seq;
            let ind : Vec<D4Line> = one_p_bsc.bsc.ind;

            if y_before > place.y || seq.len() < 4{
                d4shirataki.push(seq_part.clone());
                seq_part.clear()
            }

            //update seq
            let mut new_seq = seq.into_iter()
                .map(|v4| compress_bsc_coordinate(v4,compress_x) + vec4(place.x,place.y,0.,0.,))
                .collect();

            seq_part.append(&mut new_seq);
            //update ind
            for a_ind in ind{
                let new_aind = a_ind.into_iter()
                    .map(|v4| compress_bsc_coordinate(v4,compress_x) + vec4(place.x,place.y,0.,0.,))
                    .collect();
                d4shirataki.push(new_aind);
            }

            y_before = place.y;
        }

        d4shirataki.push(seq_part);

        //make smooth by bspline
        for i in 0..d4shirataki.len(){
            d4shirataki[i] = bspline_for_d4line(&d4shirataki[i]);
        }

        //convert to shirataki
        let shirataki = d4_2_d2(d4shirataki,resolution,boldness);
        shirataki
        
    }

    //convert row (more light) d4line to high (more heavy) d4line by using bspline
    fn bspline_for_d4line(row_d4line : &D4Line) -> D4Line {

        let mut new_d4line = Vec::new();

        //lowwer than 4 causes error
        if row_d4line.len() < 4{}
        else{
            //set "knots"
            let mut knots : Vec<f32> = Vec::new();

            knots.push(-2.0 as f32);
            knots.push(-2.0 as f32);

            let points_num = row_d4line.len();

            for i in 0..points_num{
                let para = (i+1) as f32 / points_num as f32 *4.0 -2.0;
                knots.push(para);
            }

            knots.push(2.0 as f32);
            knots.push(2.0 as f32);

            let degree = 3;
            let bspline = bspline::BSpline::new(degree, row_d4line.clone(), knots);

            //at here, you excute smoothing
            for i in 0..10*points_num{
                let first = bspline.knot_domain().0;
                let last = bspline.knot_domain().1;
                let para = (last - first) * (i as f32 /(10*points_num) as f32) + first;

                new_d4line.push(bspline.point(para));
            }
        }

        new_d4line
    }

    //Vec<D4line> to Vec<D2line> by add verosity to coordinate
    fn d4_2_d2(d4shirataki : Vec<D4Line>,resolution : u32, boldness : f32) -> Shirataki {
        let mut shirataki : Shirataki = Vec::new();

        for d4line in d4shirataki{
            for j in 0..(resolution+1){
                let mut d2line : D2Line = Vec::new();

                let mul = {
                    let jf = j as f32;
                    let maxf = (resolution+1) as f32;
                    jf / maxf * boldness
                };
                for i in 0..d4line.len(){
                    d2line.push(
                        vec2(
                            d4line[i].x + d4line[i].z * mul,
                            d4line[i].y + d4line[i].w * mul,
                        )
                    );
                }

                shirataki.push(d2line);
            }
        }

        shirataki
    }    

    fn compress_bsc_coordinate(v4 : Vec4,compress_x : f32) -> Vec4{
        vec4((v4.x-0.5) * compress_x + 0.5,v4.y,v4.z,v4.w)
    }
}

pub mod read_bcf{
    use crate::bsc::*;
    use std::fs::read_to_string;

    //convert String into bsc(for open ,bcf)
    fn string_2_bsc(bcf : String) -> BsCharactor{
        let lines = bcf.lines();

        let mut seq : D4Line = Vec::new();
        let mut a_ind : D4Line = Vec::new();
        let mut ind : Vec<D4Line> = Vec::new();

        let mut reach_ind : bool = false;

        

        for each_line in lines{
            if each_line == "seq"{}
            else if each_line == "ind"{
                if reach_ind{
                    ind.push(a_ind.clone());
                    a_ind.clear();
                }
                else{
                    reach_ind = true;
                }
            }
            else if reach_ind == false{
                let mut v_f_v4 : Vec<f32> = Vec::new();

                let num_strings = each_line.split_whitespace();


                for num_string in num_strings{
                    let num : f32 = num_string.parse().expect("not a number");

                    v_f_v4.push(num);
                }

                seq.push(vec4(v_f_v4[0],v_f_v4[1],v_f_v4[2],v_f_v4[3]));
            }
            else{
                let mut v_f_v4 : Vec<f32> = Vec::new();

                let num_strings = each_line.split_whitespace();


                for num_string in num_strings{
                    let num : f32 = num_string.parse().expect("not a number");

                    v_f_v4.push(num);
                }

                a_ind.push(vec4(v_f_v4[0],v_f_v4[1],v_f_v4[2],v_f_v4[3]));
            }
        }

        ind.push(a_ind.clone());

        BsCharactor { seq, ind }

    }

    pub fn read_bcf(path : &str) -> Option<BsCharactor>{
        let open = read_to_string(path);

        match open{
            Ok(bcf) => Some(string_2_bsc(bcf)),
            Err(_) => None
        }

    }
}

pub mod write_bcf{
    use crate::bsc::*;
    use std::fs::File;
    use std::io::Write;

    //convert bsc into string(for save as .bcf)
    fn bsc_2_string(bsc : BsCharactor) -> String{
        let mut str_out = "".to_string();

        str_out = format!("{}{}",str_out,"seq");
        for v4 in bsc.seq{
            str_out = format!("{}\n{} {} {} {}",str_out,v4.x,v4.y,v4.z,v4.w);
        }
        for i in 0..bsc.ind.len(){
            str_out = format!("{}\n{}",str_out,"ind");
            for v4 in bsc.ind[i].clone(){
                str_out = format!("{}\n{} {} {} {}",str_out,v4.x,v4.y,v4.z,v4.w);
            }
        }
        
        str_out
    }

    pub fn write_bcf(bsc : BsCharactor, path : &str){
        let bcf = bsc_2_string(bsc);

        let mut file = File::create(path).unwrap();
                    let _ = file.write_all(
                        bcf.as_bytes()
                    );

        println!("saved successfully")
    }
}