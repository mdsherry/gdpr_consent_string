use gdpr_consent_string::{ConsentString, Purpose};

#[derive(Debug, Copy, Clone)]
pub enum Field {
    Version,
    Created,
    LastUpdated,
    CmpId,
    CmpVersion,
    ConsentScreen,
    ConsentLanguage,
    VendorListVersion,
    Purposes,
    MaxVendorId,
    Consents,
}

#[derive(Debug)]
pub enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Op(Field, Opcode, Value),
    Not(Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
    In,
    NotIn,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(u64),
    Str(String),
    Vec(Vec<u64>),
}

impl Expr {
    pub fn eval(&self, gdpr: &ConsentString) -> bool {
        match self {
            Expr::And(left, right) => left.eval(gdpr) && right.eval(gdpr),
            Expr::Or(left, right) => left.eval(gdpr) || right.eval(gdpr),
            Expr::Op(field, opcode, val) => opcode.check(field.get(gdpr), val),
            Expr::Not(expr) => !expr.eval(gdpr),
        }
    }
}

impl Field {
    pub fn get(&self, gdpr: &ConsentString) -> Value {
        match self {
            Field::Version => Value::Int(gdpr.version as u64),
            Field::Created => Value::Int(
                (gdpr.created.timestamp() as u64) * 10
                    + (gdpr.created.timestamp_subsec_millis() / 100) as u64,
            ),
            Field::LastUpdated => Value::Int(
                (gdpr.last_updated.timestamp() as u64) * 10
                    + (gdpr.last_updated.timestamp_subsec_millis() / 100) as u64,
            ),
            Field::CmpId => Value::Int(gdpr.cmp_id as u64),
            Field::CmpVersion => Value::Int(gdpr.cmp_version as u64),
            Field::ConsentScreen => Value::Int(gdpr.consent_screen as u64),
            Field::ConsentLanguage => Value::Str(gdpr.consent_language.iter().collect()),
            Field::VendorListVersion => Value::Int(gdpr.vendor_list_version as u64),
            Field::Purposes => Value::Vec({
                let mut rv = vec![];
                if gdpr.purposes_allowed.contains(Purpose::StorageAndAccess) {
                    rv.push(1);
                }
                if gdpr.purposes_allowed.contains(Purpose::Personalization) {
                    rv.push(2);
                }
                if gdpr.purposes_allowed.contains(Purpose::AdSelection) {
                    rv.push(3);
                }
                if gdpr.purposes_allowed.contains(Purpose::ContentDelivery) {
                    rv.push(4);
                }
                if gdpr.purposes_allowed.contains(Purpose::Measurement) {
                    rv.push(5);
                }
                rv
            }),
            Field::MaxVendorId => Value::Int(gdpr.max_vendor_id as u64),
            Field::Consents => Value::Vec(
                gdpr.vendor_consents
                    .iter()
                    .enumerate()
                    .filter_map(|(id, &value)| {
                        if id > 0 && value {
                            Some(id as u64)
                        } else {
                            None
                        }
                    })
                    .collect(),
            ),
        }
    }
}

impl Opcode {
    pub fn check(&self, l: Value, r: &Value) -> bool {
        match (l, r) {
            (Value::Int(l), &Value::Int(r)) => match self {
                Opcode::Gt => l > r,
                Opcode::Ge => l >= r,
                Opcode::Lt => l < r,
                Opcode::Le => l <= r,
                Opcode::Eq => l == r,
                Opcode::Ne => l != r,
                _ => unimplemented!(),
            },
            (Value::Vec(v), &Value::Int(r)) => match self {
                Opcode::In => v.contains(&r),
                Opcode::NotIn => !v.contains(&r),
                _ => unimplemented!(),
            },
            (Value::Str(ref l), &Value::Str(ref r)) => match self {
                Opcode::Eq => l == r,
                Opcode::Ne => l != r,
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}
