use crate::{gui_game::color_transform::hsva_to_rgba, take_turn, Cell, Game, Res};

use std::time::Instant;

mod color_transform;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

const RADIUS: i32 = 15;
const RADIUS_SQUARE: i32 = RADIUS.pow(2);
const BACKGROUND_COLOR: [u8; 4] = [200, 200, 200, 255];
const FOREGROUND_COLOR: [u8; 4] = [20, 20, 200, 255];
const FOREGROUND_HIGHLIGHT_COLOR: [u8; 4] = [80, 120, 255, 255];

fn create_cell(
    buffer: &mut Vec<u8>,
    player_color: Option<[u8; 4]>,
    foreground_color: [u8; 4],
    x_offset: u32,
    y_offset: u32,
    grid_size: u32,
    buffer_width: usize,
) {
    let center = (grid_size / 2) as i32;
    let radius_square = calc_piece_radius(grid_size).pow(2) as i32;

    for x_pix in 0..grid_size {
        let x_dist = (center - x_pix as i32).pow(2);
        for y_pix in 0..grid_size {
            let y_dist = (center - y_pix as i32).pow(2);
            let color = if y_dist + x_dist > radius_square {
                foreground_color
            } else {
                match player_color {
                    None => continue,
                    Some(c) => c,
                }
            };
            let x = (x_pix + x_offset) as usize;
            let y = (y_pix + y_offset) as usize;
            let index = (buffer_width * y + x) * 4;
            buffer[index] = color[0];
            buffer[index + 1] = color[1];
            buffer[index + 2] = color[2];
            buffer[index + 3] = color[3];
        }
    }
}

fn create_background_buffer(
    size: PhysicalSize<u32>,
    game: &Game,
    player_colors: &[[u8; 4]],
    grid_size: u32,
    border_width: u32,
    border_height: u32,
) -> Vec<u8> {
    let width = game.board.width as u32;
    let height = game.board.height as u32 + 1;

    let mut buffer = vec![BACKGROUND_COLOR; (size.width * size.height) as usize].concat();

    for x in 0..width {
        let x_offset = x * grid_size + border_width;
        for y in 0..height - 1 {
            let y_offset = (y + 1) * grid_size + border_height;

            let occupance = match game.board.get_cell(x as usize, y as usize) {
                None => break,
                Some(n) => n.occupance,
            };
            let player_color = if occupance == 0 {
                None
            } else {
                Some(player_colors[occupance as usize - 1])
            };
            create_cell(
                &mut buffer,
                player_color,
                FOREGROUND_COLOR,
                x_offset,
                y_offset,
                grid_size,
                size.width as usize,
            )
        }
    }

    buffer
}

fn update_background_buffer_column(
    buffer: &mut Vec<u8>,
    old_x: u32,
    x: u32,
    size: PhysicalSize<u32>,
    game: &Game,
    player_colors: &[[u8; 4]],
    grid_size: u32,
    border_width: u32,
    border_height: u32,
) {
    let height = game.board.height as u32 + 1;

    let old_x_offset = old_x * grid_size + border_width;
    for y in 0..height - 1 {
        let y_offset = (y + 1) * grid_size + border_height;

        let occupance = match game.board.get_cell(old_x as usize, y as usize) {
            None => break,
            Some(n) => n.occupance,
        };
        let player_color = if occupance == 0 {
            None
        } else {
            Some(player_colors[occupance as usize - 1])
        };
        create_cell(
            buffer,
            player_color,
            FOREGROUND_COLOR,
            old_x_offset,
            y_offset,
            grid_size,
            size.width as usize,
        )
    }
    let x_offset = x * grid_size + border_width;
    for y in 0..height - 1 {
        let y_offset = (y + 1) * grid_size + border_height;

        let occupance = match game.board.get_cell(x as usize, y as usize) {
            None => break,
            Some(n) => n.occupance,
        };
        let player_color = if occupance == 0 {
            None
        } else {
            Some(player_colors[occupance as usize - 1])
        };
        create_cell(
            buffer,
            player_color,
            FOREGROUND_HIGHLIGHT_COLOR,
            x_offset,
            y_offset,
            grid_size,
            size.width as usize,
        )
    }
}

fn calc_grid_constants(game: &Game, size: &PhysicalSize<u32>) -> (u32, u32, u32) {
    let width = game.board.width as u32;
    let height = game.board.height as u32 + 1;
    let grid_tile_width = (size.width as f64 / width as f64).floor() as u32;
    let grid_tile_height = (size.height as f64 / height as f64).floor() as u32;
    let grid_size = grid_tile_width.min(grid_tile_height);

    let border_width = (size.width - width * grid_size) / 2;
    let border_height = (size.height - height * grid_size) / 2;

    (grid_size, border_width, border_height)
}

fn calc_piece_radius(grid_size: u32) -> u32 {
    grid_size * 6 / 10 / 2
}

pub fn gui_game(
    player_count: u32,
    connect_size: usize,
    board_width: usize,
    board_height: usize,
) -> Res<()> {
    let mut game = Game::new(player_count, connect_size, board_width, board_height);
    let mut game_over = false;

    let mut player_colors: Vec<[u8; 4]> = Vec::with_capacity(player_count as usize);
    for i in 0..player_count {
        let value = (i as f64 / player_count as f64 * 255.0).ceil() as u8;
        player_colors.push(hsva_to_rgba([value, 100, 100, 255]));
    }

    let event_loop = EventLoop::new();
    let builder = WindowBuilder::new().with_title("RustyTree");
    // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
    let window = builder.build(&event_loop).unwrap();
    let mut size = window.inner_size();
    let (mut grid_size, mut border_width, mut border_height) = calc_grid_constants(&game, &size);

    let mut background_buffer: Vec<u8> = create_background_buffer(
        size,
        &game,
        &player_colors,
        grid_size,
        border_width,
        border_height,
    );
    let mut buffer = Pixels::new(
        size.width,
        size.height,
        SurfaceTexture::new(size.width, size.height, &window),
    )
    .unwrap();

    let mut mouse_x: i32 = 0;
    let mut mouse_column: u32 = 0;
    let mut mouse_y: i32 = 0;

    let (mut old_x, mut old_y) = (0, 0);
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.set_exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(s),
                ..
            } => {
                size = s;
                (grid_size, border_width, border_height) = calc_grid_constants(&game, &size);

                background_buffer = create_background_buffer(
                    size,
                    &game,
                    &player_colors,
                    grid_size,
                    border_width,
                    border_height,
                );
                buffer.resize_surface(size.width, size.height).unwrap();
                buffer.resize_buffer(size.width, size.height).unwrap();
                buffer.frame_mut().clone_from_slice(&background_buffer);
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => {
                if !game_over {
                    match take_turn(&mut game, &mut |_: &Game| Ok(mouse_column as usize)) {
                        Err(e) => println!("{}", e),
                        Ok(s) => {
                            game_over = s;
                            update_background_buffer_column(
                                &mut background_buffer,
                                mouse_column,
                                mouse_column,
                                size,
                                &game,
                                &player_colors,
                                grid_size,
                                border_width,
                                border_height,
                            );
                            buffer.frame_mut().clone_from_slice(&background_buffer);
                            if game_over {
                                println!("Player {} won!!", game.active_player);
                            }
                        }
                    };
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                if !game_over {
                    (mouse_x, mouse_y) = ((position.x).ceil() as i32, (position.y).ceil() as i32);
                    let new_mouse_column = ((mouse_x - border_width as i32) / grid_size as i32)
                        .clamp(0, board_width as i32 - 1)
                        as u32;
                    if mouse_column != new_mouse_column {
                        update_background_buffer_column(
                            &mut background_buffer,
                            mouse_column,
                            new_mouse_column,
                            size,
                            &game,
                            &player_colors,
                            grid_size,
                            border_width,
                            border_height,
                        );
                        buffer.frame_mut().clone_from_slice(&background_buffer);
                        mouse_column = new_mouse_column;
                    }
                }
            }
            Event::MainEventsCleared => {
                // Application update code.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw, in
                // applications which do not always need to. Applications that redraw continuously
                // can just render here instead.
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in MainEventsCleared, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // let start = Instant::now();

                let frame = buffer.frame_mut();

                let radius = calc_piece_radius(grid_size) as i32;
                let radius_square = radius.pow(2);

                //CleanUp
                // //MC
                // for x_offset in -RADIUS..=RADIUS {
                //     let x = ((old_x as i32) + x_offset) as u32;
                //     if x >= size.width {
                //         continue;
                //     }
                //     let height = ((RADIUS_SQUARE - x_offset.pow(2)) as f64).sqrt() as i32;
                //     for y_offset in -height..=height {
                //         let y = ((old_y as i32) + y_offset) as u32;
                //         if y >= size.height {
                //             continue;
                //         }

                //         let index = ((size.width * y + x) * 4) as usize;
                //         frame[index] = background_buffer[index];
                //         frame[index + 1] = background_buffer[index + 1];
                //         frame[index + 2] = background_buffer[index + 2];
                //         frame[index + 3] = background_buffer[index + 3];
                //     }
                // }
                // //MouseCircle
                // for x_offset in -RADIUS..=RADIUS {
                //     let x: u32 = ((mouse_x as i32) + x_offset) as u32;
                //     if x >= size.width {
                //         continue;
                //     }
                //     let height = ((RADIUS_SQUARE - x_offset.pow(2)) as f64).sqrt() as i32;
                //     for y_offset in -height..=height {
                //         let y = ((mouse_y as i32) + y_offset) as u32;
                //         if y >= size.height {
                //             continue;
                //         }

                //         let index = ((size.width * y + x) * 4) as usize;
                //         frame[index] = FOREGROUND_HIGHLIGHT_COLOR[0];
                //         frame[index + 1] = FOREGROUND_HIGHLIGHT_COLOR[1];
                //         frame[index + 2] = FOREGROUND_HIGHLIGHT_COLOR[2];
                //         frame[index + 3] = FOREGROUND_HIGHLIGHT_COLOR[3];
                //     }
                // }

                //CleanUp
                //TP
                let top_x: i32 = old_x.clamp(
                    border_width as i32 + radius,
                    (size.width - border_width) as i32 - radius,
                );
                for x_offset in -radius..=radius {
                    let x: u32 = (top_x + x_offset) as u32;
                    let height = ((radius_square - x_offset.pow(2)) as f64).sqrt() as i32;
                    for y_offset in -height..=height {
                        let y = (((border_height + grid_size / 2) as i32) + y_offset) as u32;
                        if y >= size.height {
                            continue;
                        }

                        let index = ((size.width * y + x) * 4) as usize;
                        frame[index] = background_buffer[index];
                        frame[index + 1] = background_buffer[index + 1];
                        frame[index + 2] = background_buffer[index + 2];
                        frame[index + 3] = background_buffer[index + 3];
                    }
                }
                //TopPiece
                let top_x: i32 = mouse_x.clamp(
                    border_width as i32 + radius,
                    (size.width - border_width) as i32 - radius,
                );
                let color = player_colors[game.active_player as usize];
                for x_offset in -radius..=radius {
                    let x: u32 = (top_x + x_offset) as u32;
                    let height = ((radius_square - x_offset.pow(2)) as f64).sqrt() as i32;
                    for y_offset in -height..=height {
                        let y = (((border_height + grid_size / 2) as i32) + y_offset) as u32;
                        if y >= size.height {
                            continue;
                        }

                        let index = ((size.width * y + x) * 4) as usize;
                        frame[index] = color[0];
                        frame[index + 1] = color[1];
                        frame[index + 2] = color[2];
                        frame[index + 3] = color[3];
                    }
                }

                (old_x, old_y) = (mouse_x, mouse_y);

                // let elapsed = start.elapsed();
                // println!("Debug: {:?}", elapsed);

                // for pixel in frame.chunks_exact_mut(4) {
                //     pixel[0] = 0xff; // R
                //     pixel[1] = 0xff; // G
                //     pixel[2] = 0xff; // B
                //     pixel[3] = 0xff; // A
                // }
                buffer.render().unwrap();
            }
            _ => (),
        }
    });
}
