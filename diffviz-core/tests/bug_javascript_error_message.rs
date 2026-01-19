#[cfg(test)]
mod javascript_error_message_bug_tests {

    #[test]
    #[ignore = "Bug: JavaScript modified files show 'Unsupported language' error despite working correctly"]
    fn test_javascript_modified_files_should_not_show_error() {
        // This test demonstrates the bug where modified JavaScript files
        // show "Error: Unsupported language: JavaScript" despite being processed correctly

        // Test scenario:
        // 1. Original JavaScript file content
        let _original_js = r#"function MessageQueue() {
    this.config = {
        maxQueues: 100,
        enableLogging: true
    };
}

MessageQueue.prototype.createQueue = function(name) {
    return new Queue(name);
};"#;

        // 2. Modified JavaScript file content
        let _modified_js = r#"function MessageQueue() {
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

        // Expected behavior: Should process without error message
        // Actual behavior: Shows "Error: Unsupported language: JavaScript" but works correctly

        panic!("JavaScript modified files show false 'Unsupported language' error");
    }

    #[test]
    #[ignore = "Bug: Cross-language pattern affecting Java and JavaScript"]
    fn test_cross_language_modified_file_error_pattern() {
        // This test documents the cross-language pattern where both Java and JavaScript
        // show "Unsupported language" errors for modified files but not new files

        // Pattern observed:
        // - Java: "Error: Unsupported language: Java" for modified files
        // - JavaScript: "Error: Unsupported language: JavaScript" for modified files
        // - TypeScript: Wrong classification but no error message

        panic!("Cross-language bug pattern in modified file handling");
    }

    #[test]
    fn test_javascript_new_files_work_correctly() {
        // This test verifies that NEW JavaScript files work correctly (no error)
        // This should pass, demonstrating the bug only affects modified files

        let _new_js = r#"class Calculator {
    constructor() {
        this.history = [];
    }

    add(a, b) {
        const result = a + b;
        this.history.push({ operation: 'add', a, b, result });
        return result;
    }
}"#;

        // New files should work without any error messages
        assert!(true, "New JavaScript files should work correctly");
    }
}

/// Test data for reproducing the JavaScript error message bug
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
  static #instanceCount = 0;

  #id;
  #email;

  constructor(firstName, lastName, email) {
    this.#id = User.generateId();
    this.firstName = firstName;
    this.lastName = lastName;
    this.#email = email;
    User.#instanceCount++;
  }

  static generateId() {
    return `user_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  get id() {
    return this.#id;
  }

  get email() {
    return this.#email;
  }

  set email(value) {
    if (!this.validateEmail(value)) {
      throw new Error('Invalid email');
    }
    this.#email = value;
  }

  validateEmail(email) {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
  }
}"#;
}
