use fanuc_rmi::dto;
use fanuc_real_shared::{RobotPosition, JointAngles};

fn main() {
    println!("dto::Position size: {}", std::mem::size_of::<dto::Position>());
    println!("RobotPosition size: {}", std::mem::size_of::<RobotPosition>());
    println!("dto::JointAngles size: {}", std::mem::size_of::<dto::JointAngles>());
    println!("JointAngles size: {}", std::mem::size_of::<JointAngles>());
    
    // Test serialization
    let dto_pos = dto::Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        w: 4.0,
        p: 5.0,
        r: 6.0,
        ext1: 7.0,
        ext2: 8.0,
        ext3: 9.0,
    };
    
    let robot_pos = RobotPosition {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        w: 4.0,
        p: 5.0,
        r: 6.0,
        ext1: 7.0,
        ext2: 8.0,
        ext3: 9.0,
    };
    
    let dto_bytes = bincode::serde::encode_to_vec(&dto_pos, bincode::config::standard()).unwrap();
    let robot_bytes = bincode::serde::encode_to_vec(&robot_pos, bincode::config::standard()).unwrap();
    
    println!("\ndto::Position bincode size: {}", dto_bytes.len());
    println!("RobotPosition bincode size: {}", robot_bytes.len());
    println!("\ndto::Position bincode: {:?}", dto_bytes);
    println!("RobotPosition bincode: {:?}", robot_bytes);
}

