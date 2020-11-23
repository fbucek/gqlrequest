use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use std::collections::HashMap;

/// Request for GraphQL to create JSON requets structure
///
/// ```json
/// {
///     "operationName": "createBook",
///     "variables": {
///         "book": {
///             "title": "Rocket Engineering",
///         }
///     },
///     "query": "mutation createBook($book: createBook!) {\n  createBook(book: $book) {\n    title\n }\n}\n"
/// }
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub variables: HashMap<String, Value>,
    pub query: String,
}

impl GqlRequest {
    /// Cretas new request with only one query
    pub fn new(query: &str) -> Self {
        GqlRequest {
            operation_name: None,
            variables: HashMap::new(),
            query: query.to_string(),
        }
    }

    /// Crete new request for GraphQL with anonymous query/mutation
    /// ```json, no_run
    /// {
    ///     query: "info()"
    ///     variables: "book": { "title": "Rocket Engineering" }
    /// }
    pub fn new_with_variable<T: Serialize>(query: &str, variable: &str, object: &T) -> Self {
        GqlRequest {
            operation_name: None,
            variables: [(variable.to_string(), serde_json::json!(object))]
                .iter()
                .cloned()
                .collect(),
            query: query.to_string(),
        }
    }

    /// Create new request with opetaion name
    /// ```json, no_run
    /// {
    ///     query: ""
    /// }
    pub fn new_with_op(operation_name: &str, query: &str) -> Self {
        GqlRequest {
            operation_name: Some(operation_name.to_string()),
            variables: HashMap::new(),
            query: query.to_string(),
        }
    }
    pub fn add_variable<T: Serialize>(&mut self, name: &str, object: &T) -> Result<()> {
        if self.operation_name.is_none() && !self.variables.is_empty() {
            Err(eyre::eyre!(
                "Not possible to add variable when using anonymous query/mutation"
            ))
        } else {
            let json = serde_json::json!(object);
            self.variables.insert(name.to_string(), json);
            Ok(())
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GqlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<ErrorMsg>>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorMsg {
    pub message: String,
    pub locations: Vec<Location>,
    pub path: Option<Vec<Value>>,
    pub extensions: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct Location {
    pub line: i32,
    pub column: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_add_test() {
        #[derive(Serialize)]
        struct TestQuery {
            pub title: String,
        }
        let test = TestQuery {
            title: "Rocket Engineering".to_string(),
        };

        let mut request = GqlRequest::new_with_variable("", "test", &test);
        assert!(request.add_variable("test", &test).is_err())
    }

    #[test]
    fn empty_variables_test() {
        let query = "{ apiVersion }";
        let expected_body = serde_json::json!({
            "query": query,
        });

        let request = GqlRequest::new("{ apiVersion }");
        let request = serde_json::json!(&request);
        assert_eq!(request, expected_body);
    }

    #[test]
    fn request_test() {
        #[derive(Serialize)]
        struct TestQuery {
            pub title: String,
        }
        let exp_data = r#"
        {
            "operationName": "createBook",
            "variables": {
                "book": {
                    "title": "Rocket Engineering"
                }
            },
            "query": "mutation createBook($book: createBook!) { createBook(book: $book) { title }}"
        }
        "#;

        let test_query = TestQuery {
            title: "Rocket Engineering".to_string(),
        };
        let op_name = "createBook";
        let query = "mutation createBook($book: createBook!) { createBook(book: $book) { title }}";

        let mut gql_request = GqlRequest::new_with_op(op_name, query);
        gql_request.add_variable("book", &test_query).unwrap();

        let request = serde_json::json!(gql_request);
        let expected: serde_json::Value = serde_json::from_str(exp_data).unwrap();

        assert_eq!(request["operationName"], expected["operationName"]);
        assert_eq!(request, expected);
    }

    #[test]
    fn request_anonymous_test() {
        #[derive(Serialize)]
        struct TestQuery {
            pub title: String,
        }
        let exp_data = r#"
        {
            "variables": {
                "book": {
                    "title": "Rocket Engineering"
                }
            },
            "query": "mutation ($book: createBook!) { createBook(book: $book) { title }}"
        }
        "#;

        let test_query = TestQuery {
            title: "Rocket Engineering".to_string(),
        };
        let query = "mutation ($book: createBook!) { createBook(book: $book) { title }}";
        let gql_request = GqlRequest::new_with_variable(query, "book", &test_query);

        let request = serde_json::json!(gql_request);
        let expected: serde_json::Value = serde_json::from_str(exp_data).unwrap();

        assert_eq!(request, expected);
    }

    #[test]
    fn response_test() {
        let expected = r#"{"data":{"sensor":{"createdAt":"2020-09-15T07:08:54.668686+00:00","id":"59de6057-e913-45e3-95b1-e628741443fd","location":null,"macaddress":"DC:A6:32:0B:62:37","name":"unnamed-59de6057-e913-45e3-95b1-e628741443fd","updatedAt":"2020-09-15T07:08:54.668686+00:00"}}}"#;

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Sensor {
            pub name: String,
            pub location: Option<String>,
            pub macaddress: String,
            pub created_at: String,
            pub updated_at: String,
        }

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct SensorData {
            pub sensor: Sensor,
        }

        let response: GqlResponse<SensorData> = serde_json::from_str(expected).unwrap();

        let data = response.data.unwrap();

        assert_eq!(
            data.sensor.name,
            "unnamed-59de6057-e913-45e3-95b1-e628741443fd"
        );
    }

    /// Error taken from: https://lucasconstantino.github.io/graphiql-online/
    #[test]
    fn error_response_ext_test() {
        let expected = r#"{ "errors": [ { "message": "Cannot query field \"named\" on type \"Country\". Did you mean \"name\"?", "locations": [ { "line": 34, "column": 5 } ], "extensions": { "code": "GRAPHQL_VALIDATION_FAILED" } } ] }"#;

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Country {
            name: String,
        }

        let response: GqlResponse<Country> = serde_json::from_str(expected).unwrap();

        assert!(response.data.is_none());
        assert!(response.errors.is_some());

        let errors = response.errors.unwrap();

        assert_eq!(errors.len(), 1);

        let error = errors.first().unwrap();
        assert_eq!(error.message, r#"Cannot query field "named" on type "Country". Did you mean "name"?"#);
        assert_eq!(error.locations.len(), 1);
        let location = error.locations.first().unwrap();
        assert_eq!(location.line, 34);
        assert_eq!(location.column, 5);
    }


    /// Error taken from: https://lucasconstantino.github.io/graphiql-online/
    #[test]
    fn error_response_path_test() {
        let expected = r#"{ "data": null, "errors": [ { "message": "Failed to parse \"UUID\": invalid length: expected one of [36, 32], found 7", "locations": [ { "line": 2, "column": 14 } ], "path": [ "sensor" ] } ] }"#;

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Country {
            name: String,
        }

        let response: GqlResponse<Country> = serde_json::from_str(expected).unwrap();

        assert!(response.data.is_none());
        assert!(response.errors.is_some());

        let errors = response.errors.unwrap();

        assert_eq!(errors.len(), 1);

        let error = errors.first().unwrap();
        assert_eq!(error.message, r#"Failed to parse "UUID": invalid length: expected one of [36, 32], found 7"#);
        assert_eq!(error.locations.len(), 1);
        let location = error.locations.first().unwrap();
        assert_eq!(location.line, 2);
        assert_eq!(location.column, 14);

        assert!(error.path.is_some());

    }
}
