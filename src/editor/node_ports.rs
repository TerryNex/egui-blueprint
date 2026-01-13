//! Node port definitions for all node types.
//!
//! This module contains the `get_ports_for_type` function which defines
//! the input and output ports for each node type in the blueprint system.
//!
//! # Port Types
//! - **ExecutionFlow**: Control flow connections (white arrows)
//! - **Boolean**: True/false values (red)
//! - **Integer**: Whole numbers (light blue)
//! - **Float**: Decimal numbers (green)
//! - **String**: Text values (khaki)
//! - **Array**: Collection of values (orange)
//! - **Custom**: Special types like "Any" (gray)

use crate::graph::{Port, VariableValue};
use crate::node_types::{DataType, NodeType};

/// Returns the input and output port definitions for a given node type.
///
/// # Arguments
/// * `node_type` - The type of node to get ports for
///
/// # Returns
/// A tuple of (inputs, outputs) where each is a Vec<Port>
pub fn get_ports_for_type(node_type: &NodeType) -> (Vec<Port>, Vec<Port>) {
    match node_type {
            NodeType::BlueprintFunction { name } if name == "Event Tick" => (
                vec![],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            NodeType::Branch => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Condition".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
                vec![
                    Port {
                        name: "True".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "False".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                ],
            ),
            NodeType::BlueprintFunction { name } if name == "Print String" => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "String".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("Hello".into()),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // Notes - Comment node for adding memos (no ports, just display text)
            NodeType::Notes => (
                vec![], // No inputs - text is stored in display_name
                vec![], // No outputs
            ),
            NodeType::Add => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            NodeType::Multiply => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            NodeType::ToInteger => (
                vec![Port {
                    name: "In".into(),
                    data_type: DataType::Custom("Any".into()),
                    default_value: VariableValue::Integer(0),
                }],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Integer,
                    default_value: VariableValue::Integer(0),
                }],
            ),
            NodeType::ToFloat => (
                vec![Port {
                    name: "In".into(),
                    data_type: DataType::Custom("Any".into()),
                    default_value: VariableValue::Float(0.0),
                }],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            NodeType::ToString => (
                vec![Port {
                    name: "In".into(),
                    data_type: DataType::Custom("Any".into()),
                    default_value: VariableValue::String("".into()),
                }],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
            ),
            NodeType::Divide => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(1.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            NodeType::GetVariable { .. } => (
                vec![],
                vec![Port {
                    name: "Value".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            NodeType::SetVariable { .. } => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Value".into(),
                        data_type: DataType::Custom("Any".into()),
                        default_value: VariableValue::Float(0.0),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            NodeType::Subtract => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            // Comparison nodes - output Boolean
            NodeType::Equals
            | NodeType::NotEquals
            | NodeType::GreaterThan
            | NodeType::GreaterThanOrEqual
            | NodeType::LessThan
            | NodeType::LessThanOrEqual => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Boolean,
                    default_value: VariableValue::Boolean(false),
                }],
            ),
            // Logic nodes
            NodeType::And | NodeType::Or => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Boolean,
                    default_value: VariableValue::Boolean(false),
                }],
            ),
            NodeType::Not => (
                vec![Port {
                    name: "In".into(),
                    data_type: DataType::Boolean,
                    default_value: VariableValue::Boolean(false),
                }],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Boolean,
                    default_value: VariableValue::Boolean(true),
                }],
            ),
            // For Loop
            NodeType::ForLoop => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Start".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "End".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(10),
                    },
                ],
                vec![
                    Port {
                        name: "Loop".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Index".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Done".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                ],
            ),
            // While Loop
            NodeType::WhileLoop => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Condition".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(true),
                    },
                ],
                vec![
                    Port {
                        name: "Loop".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Done".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                ],
            ),
            // Delay
            NodeType::Delay => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Duration (ms)".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(1000),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // GetTimestamp - Get current Unix timestamp
            NodeType::GetTimestamp => (
                vec![
                    Port {
                        name: "Milliseconds".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(true), // Default: 13-digit milliseconds
                    },
                ],
                vec![Port {
                    name: "Timestamp".into(),
                    data_type: DataType::Integer,
                    default_value: VariableValue::Integer(0),
                }],
            ),
            // Modulo (%)
            NodeType::Modulo => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(1),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Integer,
                    default_value: VariableValue::Integer(0),
                }],
            ),
            // Power (^)
            NodeType::Power => (
                vec![
                    Port {
                        name: "Base".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(2.0),
                    },
                    Port {
                        name: "Exponent".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(2.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            // Abs
            NodeType::Abs => (
                vec![Port {
                    name: "In".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            // Min
            NodeType::Min => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            // Max
            NodeType::Max => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            // Clamp
            NodeType::Clamp => (
                vec![
                    Port {
                        name: "Value".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "Min".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "Max".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(1.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            // Random
            NodeType::Random => (
                vec![
                    Port {
                        name: "Min".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "Max".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(1.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            // Xor
            NodeType::Xor => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Boolean,
                    default_value: VariableValue::Boolean(false),
                }],
            ),
            // Sequence - executes multiple flows in order
            NodeType::Sequence => (
                vec![Port {
                    name: "In".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
                vec![
                    Port {
                        name: "Then 0".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Then 1".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Then 2".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                ],
            ),
            // Gate - on/off flow control
            NodeType::Gate => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Open".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(true),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // Concat
            NodeType::Concat => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
            ),
            // Split
            NodeType::Split => (
                vec![
                    Port {
                        name: "String".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Delimiter".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String(",".into()),
                    },
                    Port {
                        name: "Index".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
            ),
            // Length
            NodeType::Length => (
                vec![Port {
                    name: "String".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Integer,
                    default_value: VariableValue::Integer(0),
                }],
            ),
            // Contains
            NodeType::Contains => (
                vec![
                    Port {
                        name: "String".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Substring".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Boolean,
                    default_value: VariableValue::Boolean(false),
                }],
            ),
            // Replace
            NodeType::Replace => (
                vec![
                    Port {
                        name: "String".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "From".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "To".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
            ),
            // Format
            NodeType::Format => (
                vec![
                    Port {
                        name: "Template".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("Hello {}!".into()),
                    },
                    Port {
                        name: "Arg0".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
            ),
            // StringJoin - Dynamic string concatenation with auto-expanding inputs
            NodeType::StringJoin => (
                vec![
                    Port {
                        name: "Input 0".into(),
                        data_type: DataType::Custom("Any".into()),
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Input 1".into(),
                        data_type: DataType::Custom("Any".into()),
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
            ),
            // StringBetween - Extract content between two delimiter strings
            NodeType::StringBetween => (
                vec![
                    Port {
                        name: "Source".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Before".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "After".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
            ),
            // StringTrim - Trim whitespace from string with mode options
            // Mode: 0 = Both (default), 1 = Start only, 2 = End only, 3 = All (including internal)
            NodeType::StringTrim => (
                vec![
                    Port {
                        name: "String".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("  hello world  ".into()),
                    },
                    Port {
                        name: "Mode".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0), // 0=Both, 1=Start, 2=End, 3=All
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
            ),
            // ReadInput - placeholder for future interactive input
            NodeType::ReadInput => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Prompt".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("Enter value:".into()),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Value".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
            ),
            // FileRead
            NodeType::FileRead => (
                vec![Port {
                    name: "Path".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
                vec![
                    Port {
                        name: "Content".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Success".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),
            // FileWrite
            NodeType::FileWrite => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Path".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Content".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Success".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),
            // System Control: RunCommand
            NodeType::RunCommand => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Command".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Args".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Output".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "ExitCode".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Success".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),
            // System Control: LaunchApp
            NodeType::LaunchApp => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Path".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Args".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Success".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),
            // System Control: CloseApp
            NodeType::CloseApp => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Name".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Success".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),
            // System Control: FocusWindow
            NodeType::FocusWindow => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Title".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Success".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),
            // System Control: GetWindowPosition
            NodeType::GetWindowPosition => (
                vec![Port {
                    name: "Title".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
                vec![
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Width".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Height".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Found".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),
            // System Control: SetWindowPosition
            NodeType::SetWindowPosition => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Title".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Width".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(800),
                    },
                    Port {
                        name: "Height".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(600),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Success".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),
            // Desktop Input Automation (Module A)
            // Click - Click at screen coordinates (x, y)
            NodeType::Click => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // DoubleClick - Double-click at coordinates
            NodeType::DoubleClick => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // RightClick - Right-click at coordinates
            NodeType::RightClick => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // MouseMove - Move cursor to coordinates
            NodeType::MouseMove => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // MouseDown - Press mouse button without releasing
            NodeType::MouseDown => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Button".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("left".into()),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // MouseUp - Release mouse button
            NodeType::MouseUp => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Button".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("left".into()),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // Scroll - Mouse wheel scroll
            NodeType::Scroll => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(-3),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // KeyPress - Press and release a key
            NodeType::KeyPress => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Key".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("Return".into()),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // KeyDown - Press key without releasing
            NodeType::KeyDown => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Key".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("Shift".into()),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // KeyUp - Release a pressed key
            NodeType::KeyUp => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Key".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("Shift".into()),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // TypeText - Type a string of text
            NodeType::TypeText => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Text".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("Hello World".into()),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // TypeString - Type a string by simulating individual key presses with delay
            NodeType::TypeString => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Text".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Delay".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(50),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // HotKey - Key combinations (Ctrl+C, Cmd+V, etc.)
            NodeType::HotKey => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Key".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("c".into()),
                    },
                    Port {
                        name: "Ctrl".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(true),
                    },
                    Port {
                        name: "Shift".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                    Port {
                        name: "Alt".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                    Port {
                        name: "Command".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            // === Module H: Data Operations ===

            // ArrayCreate - Create an empty array
            NodeType::ArrayCreate => (
                vec![],
                vec![Port {
                    name: "Array".into(),
                    data_type: DataType::Array,
                    default_value: VariableValue::Array(vec![]),
                }],
            ),

            // ArrayPush - Add element to end of array (execution flow)
            NodeType::ArrayPush => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Variable".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("myArray".into()),
                    },
                    Port {
                        name: "Value".into(),
                        data_type: DataType::Custom("Any".into()),
                        default_value: VariableValue::None,
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),

            // ArrayPop - Remove and return last element (execution flow)
            NodeType::ArrayPop => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Variable".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("myArray".into()),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Value".into(),
                        data_type: DataType::Custom("Any".into()),
                        default_value: VariableValue::None,
                    },
                ],
            ),

            // ArrayGet - Get element by index (pure function)
            NodeType::ArrayGet => (
                vec![
                    Port {
                        name: "Array".into(),
                        data_type: DataType::Array,
                        default_value: VariableValue::Array(vec![]),
                    },
                    Port {
                        name: "Index".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                ],
                vec![Port {
                    name: "Value".into(),
                    data_type: DataType::Custom("Any".into()),
                    default_value: VariableValue::None,
                }],
            ),

            // ArraySet - Set element by index (execution flow)
            NodeType::ArraySet => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Variable".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("myArray".into()),
                    },
                    Port {
                        name: "Index".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Value".into(),
                        data_type: DataType::Custom("Any".into()),
                        default_value: VariableValue::None,
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),

            // ArrayLength - Get length of array (pure function)
            NodeType::ArrayLength => (
                vec![Port {
                    name: "Array".into(),
                    data_type: DataType::Array,
                    default_value: VariableValue::Array(vec![]),
                }],
                vec![Port {
                    name: "Length".into(),
                    data_type: DataType::Integer,
                    default_value: VariableValue::Integer(0),
                }],
            ),

            // JSONParse - Parse JSON string (pure function)
            NodeType::JSONParse => (
                vec![Port {
                    name: "JSON".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("{}".into()),
                }],
                vec![Port {
                    name: "Value".into(),
                    data_type: DataType::Custom("Any".into()),
                    default_value: VariableValue::None,
                }],
            ),

            // JSONStringify - Convert value to JSON string (pure function)
            NodeType::JSONStringify => (
                vec![Port {
                    name: "Value".into(),
                    data_type: DataType::Custom("Any".into()),
                    default_value: VariableValue::None,
                }],
                vec![Port {
                    name: "JSON".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
            ),

            // HTTPRequest - Make HTTP request (execution flow)
            NodeType::HTTPRequest => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "URL".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("https://api.example.com".into()),
                    },
                    Port {
                        name: "Method".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("GET".into()),
                    },
                    Port {
                        name: "Body".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Response".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Success".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // Screenshot & Image Tools (Module C)
            // ScreenCapture - Capture full screen or specific display
            NodeType::ScreenCapture => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Display".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "ImagePath".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Success".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // SaveScreenshot - Save screenshot to file
            NodeType::SaveScreenshot => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "ImagePath".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Filename".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("screenshot.png".into()),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "SavedPath".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Success".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // RegionCapture - Capture a specific screen region to image
            NodeType::RegionCapture => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Width".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(200),
                    },
                    Port {
                        name: "Height".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(100),
                    },
                    Port {
                        name: "Filename".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "ImagePath".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Success".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // Image Recognition (Module D)
            // GetPixelColor - Get RGB color at screen coordinates
            NodeType::GetPixelColor => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "R".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "G".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Success".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // FindColor - Search for color in screen region
            NodeType::FindColor => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "R".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(255),
                    },
                    Port {
                        name: "G".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Tolerance".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(10),
                    },
                    Port {
                        name: "RegionX".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "RegionY".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "RegionW".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(1920),
                    },
                    Port {
                        name: "RegionH".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(1080),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Found".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // WaitForColor - Wait until color appears
            NodeType::WaitForColor => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "R".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(255),
                    },
                    Port {
                        name: "G".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Tolerance".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(10),
                    },
                    Port {
                        name: "Timeout".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(5000),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Found".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // FindImage - Template matching on screen
            NodeType::FindImage => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "ImagePath".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("template.png".into()),
                    },
                    Port {
                        name: "Tolerance".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(10),
                    },
                    Port {
                        name: "RegionX".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "RegionY".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "RegionW".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(1920),
                    },
                    Port {
                        name: "RegionH".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(1080),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Found".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // WaitForImage - Wait until image appears on screen
            NodeType::WaitForImage => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "ImagePath".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("template.png".into()),
                    },
                    Port {
                        name: "Tolerance".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(10),
                    },
                    Port {
                        name: "Timeout".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(5000),
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "X".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Y".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Found".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // ImageSimilarity - Compare two images with tolerance (pure function)
            NodeType::ImageSimilarity => (
                vec![
                    Port {
                        name: "ImagePath1".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("image1.png".into()),
                    },
                    Port {
                        name: "ImagePath2".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("image2.png".into()),
                    },
                    Port {
                        name: "Tolerance".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(10),
                    },
                ],
                vec![
                    Port {
                        name: "Similarity".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "Match".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // Constant - Simple value passthrough
            NodeType::Constant => (
                vec![Port {
                    name: "Value".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),

            // ExtractAfter - Extract N characters after a keyword
            NodeType::ExtractAfter => (
                vec![
                    Port {
                        name: "Source".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Keyword".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Length".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(10),
                    },
                ],
                vec![
                    Port {
                        name: "Result".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Found".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // ExtractUntil - Extract content from keyword until delimiter
            NodeType::ExtractUntil => (
                vec![
                    Port {
                        name: "Source".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Keyword".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Delimiter".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String(",".into()),
                    },
                ],
                vec![
                    Port {
                        name: "Result".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("".into()),
                    },
                    Port {
                        name: "Found".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // WaitForCondition - Blocks until condition becomes true
            NodeType::WaitForCondition => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Condition".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                    Port {
                        name: "Poll Interval (ms)".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(100),
                    },
                    Port {
                        name: "Timeout (ms)".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0), // 0 = no timeout
                    },
                ],
                vec![
                    Port {
                        name: "Next".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Timed Out".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
            ),

            // ForLoopAsync - For loop that waits for Continue signal before each iteration
            NodeType::ForLoopAsync => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Start".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "End".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(10),
                    },
                    Port {
                        name: "Continue".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                ],
                vec![
                    Port {
                        name: "Loop".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Index".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Done".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                ],
            ),

            _ => (vec![], vec![]),
        }
    }
