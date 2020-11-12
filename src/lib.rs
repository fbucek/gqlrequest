use color_eyre::Result;
use serde::{Serialize, Deserialize};
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
    pub errors: Option<Vec<ErrorMsg>>
}

#[derive(Debug, Deserialize)]
pub struct ErrorMsg {
    pub message: String,
    pub locations: Vec<Location>,
    pub path: Vec<String>,
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
}
