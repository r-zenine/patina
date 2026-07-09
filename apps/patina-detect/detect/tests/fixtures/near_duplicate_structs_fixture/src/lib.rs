/// Case 1 — reported: `UserRequest`/`UserRecord` share 4 of 5 distinct
/// fields (Jaccard 0.8, >= 4 shared) AND a hand-rolled `impl From` exists
/// between them. Both gates (overlap threshold, conversion evidence) pass.
pub struct UserRequest {
    pub name: String,
    pub email: String,
    pub age: u32,
    pub country: String,
}

pub struct UserRecord {
    pub name: String,
    pub email: String,
    pub age: u32,
    pub country: String,
    pub created_at: u64,
}

impl From<UserRequest> for UserRecord {
    fn from(req: UserRequest) -> Self {
        UserRecord {
            name: req.name,
            email: req.email,
            age: req.age,
            country: req.country,
            created_at: 0,
        }
    }
}

/// Case 2 — not reported: same field-overlap shape as case 1 (Jaccard 0.8,
/// 4 shared fields) but zero conversion code anywhere between the two
/// types — the conversion-evidence gate must exclude it even though the
/// overlap threshold alone would pass.
pub struct ProfileDraft {
    pub name: String,
    pub email: String,
    pub age: u32,
    pub country: String,
}

pub struct ProfileSnapshot {
    pub name: String,
    pub email: String,
    pub age: u32,
    pub country: String,
    pub locale: String,
}

/// Case 3 — not reported: a real `impl From` exists, but only 2 fields are
/// shared (below the `>= 4 shared fields` minimum) — the overlap gate must
/// exclude it regardless of the conversion evidence.
pub struct Small {
    pub id: u32,
    pub tag: String,
}

pub struct Other {
    pub id: u32,
    pub tag: String,
    pub extra_one: String,
    pub extra_two: String,
    pub extra_three: String,
}

impl From<Small> for Other {
    fn from(small: Small) -> Self {
        Other {
            id: small.id,
            tag: small.tag,
            extra_one: String::new(),
            extra_two: String::new(),
            extra_three: String::new(),
        }
    }
}
