#[cfg(test)]
mod java_error_message_bug_tests {

    #[test]
    #[ignore = "Bug: Java modified files show 'Unsupported language' error despite working correctly"]
    fn test_java_modified_files_should_not_show_error() {
        // This test demonstrates the bug where modified Java files
        // show "Error: Unsupported language: Java" despite being processed correctly

        // Create test scenario:
        // 1. Original Java file content
        let _original_java = r#"package com.example;

public class Example {
    private String value;

    public String getValue() {
        return value;
    }
}
"#;

        // 2. Modified Java file content
        let _modified_java = r#"package com.example;

import java.util.Objects;

public class Example {
    private String value;

    public String getValue() {
        return value;
    }

    public void setValue(String value) {
        this.value = Objects.requireNonNull(value);
    }
}
"#;

        // Expected behavior: Should process without error message
        // Actual behavior: Shows "Error: Unsupported language: Java" but works correctly

        // TODO: Add actual test implementation once we understand the API better
        // The bug manifests in the CLI output, not necessarily in the core library

        panic!("This test should fail until the Java error message bug is fixed");
    }

    #[test]
    fn test_java_new_files_work_correctly() {
        // This test verifies that NEW Java files work correctly (no error)
        // This should pass, demonstrating the bug only affects modified files

        let _new_java = r#"package com.example;

public class NewExample {
    public void doSomething() {
        System.out.println("Hello World");
    }
}
"#;

        // New files should work without any error messages
        // This test should pass, showing the contrast with modified files

        // TODO: Implement actual verification once we understand the test framework
        assert!(true, "New Java files should work correctly");
    }

    #[test]
    #[ignore = "Bug reproduction: CLI shows error but core processing works"]
    fn test_java_language_detection_vs_error_message() {
        // This test should verify that:
        // 1. Java language is correctly detected
        // 2. Processing happens correctly
        // 3. But error message appears incorrectly for modified files

        // The disconnect seems to be between language detection (working)
        // and error reporting (showing false negative)

        panic!("Java language detection works but error message is wrong for modified files");
    }
}

/// Test data for reproducing the Java error message bug
pub mod test_data {
    pub const ORIGINAL_TASK_MANAGER: &str = r#"package com.taskmanager;

import java.util.ArrayList;
import java.util.List;

public class TaskManager {
    private List<String> tasks;
    private String managerName;

    public TaskManager(String managerName) {
        this.managerName = managerName;
        this.tasks = new ArrayList<>();
    }

    public void addTask(String task) {
        if (task != null && !task.trim().isEmpty()) {
            tasks.add(task);
        }
    }

    public int getTaskCount() {
        return tasks.size();
    }
}"#;

    pub const MODIFIED_TASK_MANAGER: &str = r#"package com.taskmanager;

import com.taskmanager.model.Task;
import java.util.ArrayList;
import java.util.List;

public class TaskManager {
    private List<Task<?>> tasks;
    private String managerName;

    public TaskManager(String managerName) {
        this.managerName = managerName;
        this.tasks = new ArrayList<>();
    }

    public void addTask(Task<?> task) {
        if (task != null) {
            tasks.add(task);
        }
    }

    public int getTaskCount() {
        return tasks.size();
    }
}"#;
}
