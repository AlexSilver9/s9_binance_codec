use serde::{de, Deserialize, Deserializer, Serialize};
use serde::de::Error;

fn de_string_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u64>().map_err(serde::de::Error::custom)
}

fn de_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

fn de_from_str<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: de::Deserializer<'de>
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) => Ok(Some(s)),
        serde_json::Value::Bool(b) => Ok(Some(b.to_string())),
        _ => Err(Error::custom("Failed to deserialize to String or bool")),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SubscriptionRequest {
    #[serde(alias = "method")]
    pub method: String,
    #[serde(alias = "params")]
    pub params: Vec<String>,    // streams to subscribe to
    #[serde(alias = "id")]
    pub id: u64,
}

impl SubscriptionRequest {
    pub fn new(id: u64) -> Self {
        SubscriptionRequest {
            method: "SUBSCRIBE".to_string(),
            params: Vec::new(),
            id,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    pub fn add_stream(&mut self, stream: &str) {
        self.params.push(stream.to_string());
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SubscriptionResponse {
    #[serde(alias = "result")]
    pub result: Option<Vec<String>>,
    #[serde(alias = "id")]
    pub id: u64,
}

impl SubscriptionResponse {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Trade {
    #[serde(alias = "e")]
    pub event_type: String,         // Event type
    #[serde(alias = "E")]
    pub event_time: u64,            // Event time
    #[serde(alias = "s")]
    pub symbol: String,             // Symbol
    #[serde(alias = "t")]
    pub trade_id: u64,              // Trade ID
    #[serde(alias = "p", deserialize_with = "de_string_to_f64")]
    pub price: f64,                 // Price
    #[serde(alias = "q", deserialize_with = "de_string_to_f64")]
    pub quantity: f64,              // Quantity
    #[serde(alias = "T")]
    pub trade_time: u64,            // Trade time
    #[serde(alias = "m")]
    pub is_buyer_market_maker: bool, // Is the buyer the market maker?
    #[serde(alias = "M")]
    pub ignore: bool,               // Ignore
}

impl Trade {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_subscription_request_new() {
        let request = SubscriptionRequest::new(123);

        assert_eq!(request.method, "SUBSCRIBE");
        assert_eq!(request.id, 123);
        assert!(request.params.is_empty());
    }

    #[test]
    fn test_subscription_request_add_stream() {
        let mut request = SubscriptionRequest::new(456);

        request.add_stream("btcusdt@ticker");
        request.add_stream("ethusdt@depth");

        assert_eq!(request.params.len(), 2);
        assert_eq!(request.params[0], "btcusdt@ticker");
        assert_eq!(request.params[1], "ethusdt@depth");
    }

    #[test]
    fn test_subscription_request_serialization() {
        let mut request = SubscriptionRequest::new(789);
        request.add_stream("btcusdt@ticker");
        request.add_stream("ethusdt@kline_1m");

        let json = serde_json::to_string(&request).unwrap();
        let expected = r#"{"method":"SUBSCRIBE","params":["btcusdt@ticker","ethusdt@kline_1m"],"id":789}"#;

        assert_eq!(json, expected);
    }

    #[test]
    fn test_subscription_request_deserialization() {
        let json = r#"{"method":"SUBSCRIBE","params":["btcusdt@ticker"],"id":100}"#;
        let request: SubscriptionRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.method, "SUBSCRIBE");
        assert_eq!(request.params, vec!["btcusdt@ticker"]);
        assert_eq!(request.id, 100);
    }

    #[test]
    fn test_subscription_response_serialization() {
        let response = SubscriptionResponse {
            result: Some(vec!["btcusdt@ticker".to_string(), "ethusdt@depth".to_string()]),
            id: 200,
        };

        let json = serde_json::to_string(&response).unwrap();
        let expected = r#"{"result":["btcusdt@ticker","ethusdt@depth"],"id":200}"#;

        assert_eq!(json, expected);
    }

    #[test]
    fn test_subscription_response_deserialization() {
        let json = r#"{"result":["btcusdt@ticker","ethusdt@depth"],"id":300}"#;
        let response: SubscriptionResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.result, Some(vec!["btcusdt@ticker".to_string(), "ethusdt@depth".to_string()]));
        assert_eq!(response.id, 300);
    }

    #[test]
    fn test_subscription_response_with_null_method() {
        let response = SubscriptionResponse {
            result: None,
            id: 400,
        };

        let json = serde_json::to_string(&response).unwrap();
        let expected = r#"{"result":null,"id":400}"#;

        assert_eq!(json, expected);
    }

    #[test]
    fn test_subscription_response_deserialization_with_null_result() {
        let json = r#"{"result":null,"id":500}"#;
        let response: SubscriptionResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.result, None);
        assert_eq!(response.id, 500);
    }

    #[test]
    fn test_serde_aliases() {
        // Test that aliases work for deserialization
        let json = r#"{"method":"SUBSCRIBE","params":["test@stream"],"id":600}"#;
        let request: SubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "SUBSCRIBE");

        let json = r#"{"result":["test@stream"],"id":700}"#;
        let response: SubscriptionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, 700);
    }

    #[test]
    fn test_trade_serialization() {
        let expected = Trade {
            event_type: "trade".to_string(),
            event_time: 1759680390108723,
            symbol: "ETHUSDT".to_string(),
            trade_id: 2921785139,
            price: 4532.56,
            quantity: 0.0132,
            trade_time: 1759680390108254,
            is_buyer_market_maker: true,
            ignore: true,
        };
        let json = r#"{"e":"trade","E":1759680390108723,"s":"ETHUSDT","t":2921785139,"p":"4532.56000000","q":"0.01320000","T":1759680390108254,"m":true,"M":true}"#;
        let trade: Trade = serde_json::from_str(json).unwrap();
        assert_eq!(expected, trade);
    }
}