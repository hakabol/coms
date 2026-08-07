use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

use std::io;
use std::thread::sleep;
use std::time::Duration;

use std::process::Command;

use eframe::egui;
use std::sync::Mutex;
use std::sync::LazyLock;

pub struct ComsApp {
    messages: Vec<String>,
    input: String,
    send: bool,
}

impl Default for ComsApp {
    fn default() -> Self {
        Self {
            messages: vec![],
            input: String::new(),
            send: false
        }
    }
}

impl eframe::App for ComsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        let mut style = (*ctx.style()).clone();

        style.override_font_id = Some(
            egui::FontId::proportional(50.0)
        );

        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("COMS");
        });

        egui::TopBottomPanel::bottom("idk").show(ctx, |ui| {
            //ui.heading("Coms");

            egui::ScrollArea::vertical().show(ui, |ui| {
                for msg in &APP.lock().unwrap().messages.clone() {
                    ui.label(msg);
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                let mut app = APP.lock().unwrap();
                let input = ui.text_edit_singleline(&mut app.input);
                let msg = app.input.clone();

                if input.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    app.messages.push(msg.clone());
                    //app.input.clear();
                    app.send = true;
                }

                if ui.button("Send").clicked() {
                    app.messages.push(msg);
                    app.send = true;
                }
            });
        });
    }
}

pub static APP: LazyLock<Mutex<ComsApp>> = LazyLock::new(|| {Mutex::new(ComsApp::default()) });

fn get_ip() -> String{
    //let output = Command::new("curl")
    //    .arg("-4")
    //    .arg("ifconfig.me")
    //    .output()
    //    .unwrap();

    //String::from_utf8(output.stdout)
    //    .unwrap()
    //    .trim()
    //    .to_string()

    "127.0.0.1".to_string()
}

#[tokio::main]
async fn main() -> eframe::Result<()>{
    let ip = get_ip();

    print!("enter the listening port: ");
    io::stdout().flush().unwrap();

    let mut lis_ip = String::new();
    io::stdin().read_line(&mut lis_ip).unwrap();
    lis_ip = lis_ip.trim_end().to_string();

    lis_ip = ip + ":" + &lis_ip;

    println!("your socket is: {}", lis_ip);

    print!("enter the sending ip format (ip:port): ");
    io::stdout().flush().unwrap();

    let mut rec_ip = String::new();
    io::stdin().read_line(&mut rec_ip).unwrap();
    rec_ip = rec_ip.trim_end().to_string();

    rec_ip = "127.0.0.1:3000".to_string();
    lis_ip = "127.0.0.1:3001".to_string();

    tokio::spawn(
            reciver(lis_ip.clone()),
    );
    tokio::spawn(
            sender(rec_ip.clone()),
    );

    eframe::run_native("coms", eframe::NativeOptions::default(), Box::new(|_cc| Ok(Box::new(ComsApp::default()))))

}

async fn sender(ip: String){
    let mut stream;

    {
        APP.lock().unwrap().messages = vec!["waiting for connection......".to_string()];
    }

    sleep(Duration::from_millis(30));

    loop{
        match TcpStream::connect(&ip){
            Ok(s) =>{
                stream = s;
                println!("connected");
                {
                    APP.lock().unwrap().messages = vec![];
                }
                break;
            }
            Err(_) => {
                sleep(Duration::from_millis(30));
            }
        }
    }

    loop{
            {
            let mut app = APP.lock().unwrap();
            if app.send{
                let msg = app.input.clone();

                println!("{msg}");
                stream.write_all(&msg.into_bytes()).unwrap();
                app.send = false;
                app.input.clear();
            }
        }
        sleep(Duration::from_millis(20));
    }
}

async fn reciver(ip: String){
    let listener = TcpListener::bind(ip).unwrap();

    for stream in listener.incoming(){
        match stream{
            Ok(mut stream) => {
                loop{
                    let mut buffer = [0; 512];
                    match stream.read(&mut buffer){
                        Ok(n) if n != 0 => {
                            let msg = String::from_utf8_lossy(&buffer[..n]).to_string();
                            Command::new("notify-send").args(["coms", &format!("msg: {}, from: {}", msg, stream.peer_addr().unwrap())]).status().unwrap();
                            APP.lock().unwrap().messages.push(msg);
                            //println!("{msg}")
                        }

                        Ok(_) => {}
                        Err(e) => println!("Error reading: {}", e)
                    }
                }
            }

            Err(e) => println!("Error reading: {}", e)
        }
    }
}
