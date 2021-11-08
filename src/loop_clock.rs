use std::time::{Duration, Instant};

pub struct LoopClock {
    target_frametime: Duration,
    frame_count: i32,
    accum_time: f32,
    last_update_inst: Instant,
    last_frame_inst: Instant,
}

impl LoopClock {
    pub fn start_clock(fps: f64) -> Self {
        let target_frametime = Duration::from_secs_f64(1.0 / fps);
        let frame_count = 0;
        let accum_time = 0.0;
        let last_update_inst = Instant::now();
        let last_frame_inst = Instant::now();

        LoopClock {
            target_frametime,
            frame_count,
            accum_time,
            last_update_inst,
            last_frame_inst,
        }
    }

    pub fn get_wait_duration(&mut self) -> Option<Instant> {
        let target_frametime = Duration::from_secs_f64(1.0 / 60.0);
        let time_since_last_frame = self.last_update_inst.elapsed();
        if time_since_last_frame >= target_frametime {
            self.last_update_inst = Instant::now();
            None
        } else {
            Some(Instant::now() + self.target_frametime - time_since_last_frame)
        }
    }

    pub fn tick(&mut self) -> Option<f32> {
        self.accum_time += self.last_frame_inst.elapsed().as_secs_f32();
        self.last_frame_inst = Instant::now();
        (self.frame_count == 100).then(|| {
            let average_frametime = self.accum_time * 1000.0 / self.frame_count as f32;
            self.accum_time = 0.0;
            self.frame_count = 0;
            average_frametime
        })
    }
}
