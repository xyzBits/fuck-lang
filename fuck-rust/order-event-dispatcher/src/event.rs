// 带数据的枚举 enum with payload
#[derive(Debug, Clone)]
pub enum OrderEvent {
    New { id: u64, price: u64, qty: u64 },
    Cancel { id: u64 },
    Trade { id: u64, qty: u64 },
}

impl OrderEvent {
    // &self 是一个缩写
    // self: &Self
    // self: &OrderEvent
    // pub fn order_id(self: &Self) -> u64 {
    // pub fn order_id(self: &OrderEvent) -> u64 {
    pub fn order_id(&self) -> u64 {
        match self {
            Self::New { id, .. } | Self::Cancel { id } | Self::Trade { id, .. } => *id,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn order_id_extraction() {
        let new = OrderEvent::New {
            id: 1,
            price: 100,
            qty: 10,
        };

        let cancel = OrderEvent::Cancel { id: 2 };
        let trade = OrderEvent::Trade { id: 3, qty: 3 };

        assert_eq!(new.order_id(), 1);
        assert_eq!(cancel.order_id(), 2);
        assert_eq!(trade.order_id(), 3);
    }
}
