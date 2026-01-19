//! TypeScript-specific demonstration of the ReviewableDiff pipeline
//!
//! This example shows how to:
//! 1. Parse TypeScript source code into AST trees
//! 2. Detect changes using TypeScript-specific strategies  
//! 3. Expand changes with semantic context including decorators
//! 4. Convert to structured ReviewableDiff objects
//! 5. Display results with debug formatting
//!
//! Note: This demo uses the TypeScript parser with enhanced features including:
//! - Decorator detection (@Component, @Injectable, etc.)
//! - Interface and type analysis
//! - Generic type parameter tracking
//! - Export/import statement parsing
//! - Method visibility detection
//!
//! Run with: cargo run --example typescript_reviewable_diff_demo

use std::error::Error;
use tree_sitter::Parser;

use diffviz_core::common::{LanguageParser, ProgrammingLanguage};
use diffviz_core::{
    ast_diff::SourceCode, parsers::TypeScriptParser, renderable_diff::RenderableDiff,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Original version - Basic React component with class-based approach
const OLD_CODE: &str = r#"import React, { Component } from 'react';
import { User } from './types';

interface Props {
    user: User;
    onUpdate: (user: User) => void;
}

interface State {
    isEditing: boolean;
    formData: Partial<User>;
}

class UserProfile extends Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            isEditing: false,
            formData: {}
        };
    }

    handleEdit = () => {
        this.setState({
            isEditing: true,
            formData: { ...this.props.user }
        });
    }

    handleSave = () => {
        const { onUpdate } = this.props;
        const { formData } = this.state;
        
        if (this.isValidUser(formData)) {
            onUpdate(formData as User);
            this.setState({ isEditing: false, formData: {} });
        }
    }

    private isValidUser(user: Partial<User>): user is User {
        return !!(user.name && user.email);
    }

    render() {
        const { user } = this.props;
        const { isEditing, formData } = this.state;

        if (isEditing) {
            return (
                <form onSubmit={this.handleSave}>
                    <input 
                        value={formData.name || ''} 
                        onChange={e => this.setState({
                            formData: { ...formData, name: e.target.value }
                        })}
                    />
                    <button type="submit">Save</button>
                </form>
            );
        }

        return (
            <div>
                <h2>{user.name}</h2>
                <p>{user.email}</p>
                <button onClick={this.handleEdit}>Edit</button>
            </div>
        );
    }
}

export default UserProfile;
"#;

/// Refactored version - Modern functional component with hooks and decorators
const NEW_CODE: &str = r#"import React, { useState, useCallback, useMemo } from 'react';
import { User, ValidationResult, ApiResponse } from './types';
import { useNotification } from './hooks/useNotification';
import { userValidationSchema } from './validation';

interface UserProfileProps {
    user: User;
    onUpdate: (user: User) => Promise<ApiResponse<User>>;
    onError?: (error: Error) => void;
    readonly?: boolean;
}

interface EditingState {
    isEditing: boolean;
    formData: Partial<User>;
    errors: Record<string, string>;
    isSubmitting: boolean;
}

// Custom hook for form state management
const useUserForm = (initialUser: User) => {
    const [state, setState] = useState<EditingState>({
        isEditing: false,
        formData: {},
        errors: {},
        isSubmitting: false
    });

    const updateField = useCallback((field: keyof User, value: string) => {
        setState(prev => ({
            ...prev,
            formData: { ...prev.formData, [field]: value },
            errors: { ...prev.errors, [field]: '' } // Clear error on change
        }));
    }, []);

    const validateForm = useCallback((data: Partial<User>): ValidationResult => {
        return userValidationSchema.validate(data);
    }, []);

    const resetForm = useCallback(() => {
        setState({
            isEditing: false,
            formData: {},
            errors: {},
            isSubmitting: false
        });
    }, []);

    return {
        state,
        updateField,
        validateForm,
        resetForm,
        startEditing: () => setState(prev => ({
            ...prev,
            isEditing: true,
            formData: { ...initialUser }
        })),
        setSubmitting: (isSubmitting: boolean) => setState(prev => ({
            ...prev,
            isSubmitting
        })),
        setErrors: (errors: Record<string, string>) => setState(prev => ({
            ...prev,
            errors
        }))
    };
};

// Validation decorator for form handlers
function validateInput<T extends any[]>(
    target: any,
    propertyName: string,
    descriptor: TypedPropertyDescriptor<(...args: T) => any>
) {
    const method = descriptor.value!;
    
    descriptor.value = function (this: any, ...args: T) {
        const [formData] = args;
        const validation = userValidationSchema.validate(formData);
        
        if (!validation.isValid) {
            this.formState.setErrors(validation.errors);
            return;
        }
        
        return method.apply(this, args);
    };
}

// Performance monitoring decorator
function measurePerformance(
    target: any,
    propertyName: string,
    descriptor: TypedPropertyDescriptor<Function>
) {
    const method = descriptor.value!;
    
    descriptor.value = function (this: any, ...args: any[]) {
        const start = performance.now();
        const result = method.apply(this, args);
        const end = performance.now();
        
        console.log(`${propertyName} took ${end - start} milliseconds`);
        return result;
    };
}

const UserProfile: React.FC<UserProfileProps> = ({ 
    user, 
    onUpdate, 
    onError,
    readonly = false 
}) => {
    const formState = useUserForm(user);
    const { showNotification } = useNotification();
    
    // Memoized validation to prevent unnecessary re-renders
    const validationErrors = useMemo(() => {
        if (!formState.state.isEditing) return {};
        const validation = formState.validateForm(formState.state.formData);
        return validation.isValid ? {} : validation.errors;
    }, [formState.state.formData, formState.state.isEditing, formState.validateForm]);

    @measurePerformance
    @validateInput
    const handleSave = useCallback(async (formData: Partial<User>) => {
        try {
            formState.setSubmitting(true);
            
            const response = await onUpdate(formData as User);
            
            if (response.success) {
                showNotification('User updated successfully', 'success');
                formState.resetForm();
            } else {
                throw new Error(response.error || 'Update failed');
            }
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            showNotification(errorMessage, 'error');
            onError?.(error as Error);
        } finally {
            formState.setSubmitting(false);
        }
    }, [onUpdate, onError, showNotification, formState]);

    const handleCancel = useCallback(() => {
        formState.resetForm();
    }, [formState]);

    const handleFieldChange = useCallback((field: keyof User) => 
        (e: React.ChangeEvent<HTMLInputElement>) => {
            formState.updateField(field, e.target.value);
        }, [formState]
    );

    // Render editing form
    if (formState.state.isEditing) {
        return (
            <form onSubmit={(e) => {
                e.preventDefault();
                handleSave(formState.state.formData);
            }}>
                <div className="form-field">
                    <label htmlFor="name">Name</label>
                    <input
                        id="name"
                        type="text"
                        value={formState.state.formData.name || ''}
                        onChange={handleFieldChange('name')}
                        disabled={formState.state.isSubmitting}
                        aria-invalid={!!validationErrors.name}
                        aria-describedby={validationErrors.name ? 'name-error' : undefined}
                    />
                    {validationErrors.name && (
                        <span id="name-error" className="error">
                            {validationErrors.name}
                        </span>
                    )}
                </div>

                <div className="form-field">
                    <label htmlFor="email">Email</label>
                    <input
                        id="email"
                        type="email"
                        value={formState.state.formData.email || ''}
                        onChange={handleFieldChange('email')}
                        disabled={formState.state.isSubmitting}
                        aria-invalid={!!validationErrors.email}
                        aria-describedby={validationErrors.email ? 'email-error' : undefined}
                    />
                    {validationErrors.email && (
                        <span id="email-error" className="error">
                            {validationErrors.email}
                        </span>
                    )}
                </div>

                <div className="form-actions">
                    <button 
                        type="submit" 
                        disabled={formState.state.isSubmitting || Object.keys(validationErrors).length > 0}
                    >
                        {formState.state.isSubmitting ? 'Saving...' : 'Save'}
                    </button>
                    <button 
                        type="button" 
                        onClick={handleCancel}
                        disabled={formState.state.isSubmitting}
                    >
                        Cancel
                    </button>
                </div>
            </form>
        );
    }

    // Render read-only view
    return (
        <div className="user-profile">
            <div className="user-info">
                <h2 className="user-name">{user.name}</h2>
                <p className="user-email">{user.email}</p>
                {user.bio && (
                    <p className="user-bio">{user.bio}</p>
                )}
            </div>
            
            {!readonly && (
                <div className="user-actions">
                    <button 
                        onClick={formState.startEditing}
                        className="btn btn-primary"
                    >
                        Edit Profile
                    </button>
                </div>
            )}
        </div>
    );
};

export default UserProfile;
export type { UserProfileProps };
"#;

fn main() -> Result<(), Box<dyn Error>> {
    println!("⚡ TypeScript Semantic Code Review Demo");
    println!();

    // Configuration
    let language = ProgrammingLanguage::TypeScript;

    // Setup source code objects
    let old_source = SourceCode::new(OLD_CODE);
    let new_source = SourceCode::new(NEW_CODE);

    // Parse AST trees
    let typescript_parser = TypeScriptParser::new();
    let mut ts_parser = Parser::new();
    ts_parser.set_language(typescript_parser.get_language())?;

    let old_tree = ts_parser
        .parse(OLD_CODE, None)
        .ok_or("Failed to parse old code")?;
    let new_tree = ts_parser
        .parse(NEW_CODE, None)
        .ok_or("Failed to parse new code")?;

    let old_semantic_tree = typescript_parser
        .build_semantic_tree(&old_tree, OLD_CODE)
        .map_err(|e| format!("Failed to build old semantic tree: {e}"))?;
    let new_semantic_tree = typescript_parser
        .build_semantic_tree(&new_tree, NEW_CODE)
        .map_err(|e| format!("Failed to build new semantic tree: {e}"))?;

    // Build semantic pairs and convert to reviewable diffs
    let semantic_pairs = build_semantic_pairs(
        &old_semantic_tree,
        &new_semantic_tree,
        &old_source,
        &new_source,
        &typescript_parser,
    )
    .map_err(|e| format!("Failed to build semantic pairs: {e}"))?;

    let semantic_reviewable_diffs =
        semantic_pairs_to_reviewable_diffs(&semantic_pairs, language, &old_source, &new_source);

    // Filter out changes with no visible content
    let visible_changes: Vec<&_> = semantic_reviewable_diffs
        .iter()
        .filter(|reviewable_diff| {
            let renderable: RenderableDiff = (*reviewable_diff).into();
            let visible_lines = renderable
                .lines
                .iter()
                .filter(|line| !line.should_fold())
                .count();
            visible_lines > 0
        })
        .collect();

    for (i, reviewable_diff) in visible_changes.iter().enumerate() {
        // Convert to RenderableDiff for display
        let renderable: RenderableDiff = (*reviewable_diff).into();

        let changed_lines = renderable
            .lines
            .iter()
            .filter(|line| line.has_changes())
            .count();
        let hidden_lines = renderable
            .lines
            .iter()
            .filter(|line| line.should_fold())
            .count();

        println!(
            "\n🔸 Change {} of {}: {}",
            i + 1,
            visible_changes.len(),
            renderable.metadata.boundary_name
        );

        if changed_lines > 0 {
            println!("   📊 {changed_lines} changed lines, {hidden_lines} hidden lines");
        }

        // Display source code with syntax highlighting
        println!("   📝 Source Code:");

        let mut hidden_count = 0;
        for line in &renderable.lines {
            if line.should_fold() {
                hidden_count += 1;
                continue;
            }

            if hidden_count > 0 {
                println!("   \x1b[37m  ... {hidden_count} lines hidden ...\x1b[0m");
                hidden_count = 0;
            }

            let (prefix, color) = line.get_display_style();
            println!("   {}{} {}\x1b[0m", color, prefix, line.content);
        }

        if hidden_count > 0 {
            println!("   \x1b[37m  ... {hidden_count} lines hidden ...\x1b[0m");
        }
    }

    println!(
        "\n✅ TypeScript Review complete: {} changes detected",
        visible_changes.len()
    );

    println!("\n🎯 TypeScript-specific features demonstrated:");
    println!("   • Decorator detection (@measurePerformance, @validateInput)");
    println!("   • Interface analysis (Props, State, UserProfileProps)");
    println!("   • Generic type parameters (Component<Props, State>)");
    println!("   • Hook function analysis (useState, useCallback, useMemo)");
    println!("   • Import/export statement parsing");
    println!("   • Method visibility detection (private, public)");
    println!("   • Arrow function vs regular function distinction");
    println!("   • Type annotation analysis (React.FC<UserProfileProps>)");

    Ok(())
}
