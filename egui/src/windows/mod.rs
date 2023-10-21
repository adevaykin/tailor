use egui::Context;

pub mod about;

#[derive(Default)]
pub struct Windows {
    pub about: about::AboutWindow,
}

impl Windows {
    pub fn draw(&mut self, ctx: &Context) {
        self.about.show(ctx);
    }
}