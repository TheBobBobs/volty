pub fn if_false(b: &bool) -> bool {
    !b
}

pub fn if_option_false(b: &Option<bool>) -> bool {
    *b != Some(true)
}

pub fn if_zero_u32(t: &u32) -> bool {
    *t == 0
}
