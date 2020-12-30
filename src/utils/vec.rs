pub trait RemoveItem<T> {
    // TODO: Maybe use impl Into<T> somehow?
    fn remove_one_item(&mut self, item: &T);
}

impl<T: std::cmp::PartialEq> RemoveItem<T> for Vec<T> {
    fn remove_one_item(&mut self, item: &T) {
        if let Some(pos) = self.iter().position(|x| *x == *item) {
            self.remove(pos);
        }
    }
}
