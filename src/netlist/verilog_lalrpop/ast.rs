use std::collections::{BTreeMap, HashMap};
use std::num::ParseIntError;
use std::ops::Range;
use std::str::FromStr;
use bitvec::prelude::Lsb0;
use bitvec::view::BitView;
use crate::logic::LogicVec1;

pub type Map<K, V> = BTreeMap<K, V>;

pub type Ident = String;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Primary {
    Number(Number),
    StringLit(String),
    NetSlice(NetSlice),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Literal {
    Number(Number),
    StringLit(String),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Expr {
    Primary(Primary),
    Concatenation(Vec<Expr>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ConstExpr {
    Primary(Literal),
    Concatenation(Vec<ConstExpr>),
}

impl ConstExpr {
    pub fn eval(&self) -> Option<Number> {
        match &self {
            ConstExpr::Primary(Literal::Number(n)) => Some(n.clone()),
            _ => None,
        }
    }
}

// Wires and expressions can be more than 64 bits
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
struct NumberValue(usize);

impl From<&NumberValue> for u64 {
    fn from(value: &NumberValue) -> Self {
        value.0 as u64
    }
}

impl TryFrom<&NumberValue> for u32 {
    type Error = ();

    fn try_from(value: &NumberValue) -> Result<Self, Self::Error> {
        Ok(value.0 as u32)
    }
}

impl NumberValue {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseIntError> {
        usize::from_str_radix(s, radix).map(|v| Self(v))
    }
    pub fn bit(&self, idx: SizeType) -> bool {
        self.0.view_bits::<Lsb0>()[idx as usize]
    }
}

// should be more than enough in practice
pub type SizeType = u32;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Default)]
pub struct Number {
    pub size: Option<SizeType>,
    value: NumberValue,
}

impl Number {
    pub fn from_str_radix(s: &str, radix: u32, size: Option<SizeType>) -> Result<Self, ParseIntError> {
        let value = NumberValue::from_str_radix(s, radix)?;
        Ok(Self { size, value })
    }
    pub fn bit(&self, idx: SizeType) -> bool {
        match self.size {
            Some(w) if idx >= w => false,
            _ => self.value.bit(idx),
        }
    }
}

impl TryFrom<&Number> for SizeType {
    type Error = ();

    fn try_from(value: &Number) -> Result<Self, Self::Error> {
        SizeType::try_from(&value.value)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Default, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum NetType {
    #[default]
    Wire
}

impl From<Option<&str>> for NetType {
    fn from(value: Option<&str>) -> Self {
        value.and_then(|s| Self::from_str(s).ok()).unwrap_or_default()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NetSlice(pub Ident, pub Option<Slice>);

pub(crate) type Attributes = Map<Ident, Expr>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModuleDeclaration {
    pub attrs: Attributes,
    pub name: Ident,
    pub ports: Vec<Port>,
    pub body: Vec<ModuleItem>,
}


impl ModuleDeclaration {
    pub fn net_decls(&self) -> HashMap<String, (SizeType, Attributes)> {
        self.body.iter().filter_map(|i| match i {
            ModuleItem::NetDeclaration(attrs, net_type, slice, idents) =>
                Some(
                    idents.iter().map(|i|
                        (i.to_string(), (slice.as_ref().map_or(1, |slice| slice.width()), attrs.clone()))
                    )),
            _ => None,
        }).flatten().collect()
    }
    pub fn port_decls(&self) -> HashMap<String, (SizeType, Attributes, Direction)> {
        self.body.iter().filter_map(|i| match i {
            ModuleItem::PortDeclaration(attrs, slice, idents, dir) =>
                Some(
                    idents.iter().map(|i|
                        (i.to_string(), (slice.as_ref().map_or(1, |slice| slice.width()), attrs.clone(), dir.clone()))
                    )),
            _ => None,
        }).flatten().collect()
    }

    // assuming default value of signals is Z
    pub fn continuous_assigns(&self, net_widths: HashMap<String, SizeType>) -> HashMap<String, LogicVec1> {
        let ret = HashMap::new();
        let assigns = self.body.iter().filter_map(|i| match i {
            ModuleItem::ContinuousAssign(assigns) => Some(assigns),
            _ => None,
        }).flatten();
        // for a in assigns {
        //     match (&a.0, &a.1) {
        //         (
        //             LValue::NetSlice(NetSlice(ident, slice)),
        //             Expr::Primary(Primary::Number(num))) => {
        //             for idx in slice.unwrap_or_default().range() {
        //                 // bit_assigns.push(SizeType::from(slice)
        //             }
        //         }
        //         (_, _) => todo!("Not implemented!")
        //     }
        // }
        ret
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum LValue {
    NetSlice(NetSlice),
    Concatenation(Vec<LValue>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NetAssignment(pub LValue, pub Expr);

#[derive(Clone, Debug, Eq, Hash, PartialEq, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Direction {
    Input,
    Output,
    InOut,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Slice(pub Box<Expr>, pub Option<Box<Expr>>);

impl Slice {

}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Default)]
pub struct ConstRange(pub SizeType, pub SizeType);

impl ConstRange {
    #[inline(always)]
    pub fn hi(&self) -> SizeType {
        self.0
    }
    #[inline(always)]
    pub fn lo(&self) -> SizeType {
        self.1
    }
    #[inline(always)]
    pub fn width(&self) -> SizeType {
        self.hi().max(self.lo()) - self.hi().min(self.lo()) + 1
    }
    #[inline(always)]
    pub fn range(&self) -> Range<SizeType> {
        self.hi()..self.lo()
    }
    pub fn from_const_expr(start: &ConstExpr, end: &ConstExpr) -> Result<Self, ()> {
        let do_eval = |s: &ConstExpr| s.eval().and_then(|n| SizeType::try_from(&n).ok());
        match (do_eval(start), do_eval(end)) {
            (Some(start), Some(end)) => {
                Ok(Self(SizeType::from(start), SizeType::from(end)))
            }
            _ => Err(())
        }
    }
}

pub type Port = String;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NamedPortConnection(pub Ident, pub Expr);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Connections {
    Ordered(Vec<Expr>),
    Named(Vec<(Ident, Expr)>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HierarchicalInstance(pub Ident, pub Connections);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ModuleItem {
    PortDeclaration(Attributes, Option<ConstRange>, Vec<Ident>, Direction),
    NetDeclaration(Attributes, NetType, Option<ConstRange>, Vec<Ident>),
    ModuleInstantiation(Attributes, Ident, Vec<HierarchicalInstance>),
    ContinuousAssign(Vec<NetAssignment>),
}
