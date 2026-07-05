//! JavaScript parser integration tests.
//!
//! These tests verify that `JavaScriptParser` no longer returns
//! `SemanticError::UnsupportedLanguage`. The original bug was that
//! `build_semantic_tree` for JavaScript returned an error despite the language
//! being syntactically parsed correctly. With the `JavaScriptDescriptor`-based
//! implementation, the method now succeeds.

#[cfg(test)]
mod javascript_error_message_bug_tests {
    use diffviz_core::common::LanguageParser;
    use diffviz_core::parsers::JavaScriptParser;

    #[test]
    fn test_javascript_modified_files_should_not_show_error() {
        // Verify that building a semantic tree for JavaScript code no longer
        // returns UnsupportedLanguage or any other error.
        let parser = JavaScriptParser::new();

        let modified_js = r#"function MessageQueue() {
    this.config = {
        maxQueues: 100,
        enableLogging: true,
        enableMetrics: false,
        maxMessageSize: 65536
    };

    this.metrics = {
        totalMessages: 0,
        errors: 0
    };
}

MessageQueue.prototype.createQueue = function(name) {
    return new Queue(name);
};"#;

        let tree = parser
            .try_parse(modified_js)
            .expect("JavaScript parse must succeed");
        let result = parser.build_semantic_tree(&tree, modified_js);
        assert!(
            result.is_ok(),
            "build_semantic_tree must not return UnsupportedLanguage for JavaScript: {result:?}"
        );
    }

    #[test]
    fn test_cross_language_modified_file_error_pattern() {
        // Verify that both TypeScript and JavaScript parsers successfully build
        // semantic trees — the cross-language error pattern observed was that
        // build_semantic_tree errored for both.
        use diffviz_core::parsers::TypeScriptParser;

        let js_parser = JavaScriptParser::new();
        let ts_parser = TypeScriptParser::new();

        let js_code = r#"class Calculator {
    add(a, b) { return a + b; }
}"#;

        let ts_code = r#"class Calculator {
    add(a: number, b: number): number { return a + b; }
}"#;

        for (parser, code) in [
            (
                &js_parser as &dyn diffviz_core::common::LanguageParser,
                js_code,
            ),
            (
                &ts_parser as &dyn diffviz_core::common::LanguageParser,
                ts_code,
            ),
        ] {
            let tree = parser.try_parse(code).expect("parse must succeed");
            let result = parser.build_semantic_tree(&tree, code);
            assert!(
                result.is_ok(),
                "build_semantic_tree must succeed: {result:?}"
            );
        }
    }

    #[test]
    fn test_javascript_new_files_work_correctly() {
        let parser = JavaScriptParser::new();

        let new_js = r#"class Calculator {
    constructor() {
        this.history = [];
    }

    add(a, b) {
        const result = a + b;
        this.history.push({ operation: 'add', a, b, result });
        return result;
    }
}"#;

        let tree = parser.try_parse(new_js).expect("parse must succeed");
        let semantic_tree = parser
            .build_semantic_tree(&tree, new_js)
            .expect("build_semantic_tree must succeed for JavaScript");
        assert!(!semantic_tree.root.children.is_empty());
    }
}

/// Test data retained for external use.
pub mod test_data {
    pub const ORIGINAL_MESSAGE_QUEUE_JS: &str = r#"const MessageQueue = (function() {
  'use strict';

  function MessageQueueManager() {
    this.config = {
      maxQueues: 100,
      enableLogging: true,
      retryAttempts: 3,
      retryDelay: 1000
    };
  }

  return {
    getInstance: function() {
      return new MessageQueueManager();
    }
  };
})();"#;

    pub const MODIFIED_MESSAGE_QUEUE_JS: &str = r#"const MessageQueue = (function() {
  'use strict';

  function MessageQueueManager() {
    this.config = {
      maxQueues: 100,
      enableLogging: true,
      retryAttempts: 3,
      retryDelay: 1000,
      enableMetrics: false,
      maxMessageSize: 65536
    };

    this.metrics = {
      totalMessages: 0,
      totalQueues: 0,
      errors: 0,
      startTime: Date.now()
    };
  }

  return {
    getInstance: function() {
      return new MessageQueueManager();
    }
  };
})();"#;

    pub const ES6_CLASS_EXAMPLE: &str = r#"class User {
  #id;
  #email;

  constructor(firstName, lastName, email) {
    this.#id = User.generateId();
    this.firstName = firstName;
    this.lastName = lastName;
    this.#email = email;
  }

  static generateId() {
    return `user_${Date.now()}`;
  }

  get email() {
    return this.#email;
  }
}"#;
}
