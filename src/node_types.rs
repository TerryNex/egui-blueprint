use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataType {
    ExecutionFlow,
    Boolean,
    Integer,
    Float,
    String,
    Vector3,
    /// Array type for Module H: Data Operations
    Array,
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    BlueprintFunction {
        name: String,
    },
    Branch,
    ForLoop,
    WhileLoop,
    GetVariable {
        name: String,
    },
    SetVariable {
        name: String,
    },
    /// Comment/memo node - no execution, just for notes
    Notes,
    // Math operations
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Abs,
    Min,
    Max,
    Clamp,
    Random,
    /// Constant value output - outputs the input value directly
    Constant,
    // Comparison operations
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    // Logic operations
    And,
    Or,
    Not,
    Xor,
    // Control Flow
    Sequence,
    Gate,
    // String operations
    Concat,
    Split,
    Length,
    Contains,
    Replace,
    Format,
    /// Dynamic string concatenation - auto-expands inputs when connected
    StringJoin,
    /// Extract content between two delimiter strings
    StringBetween,
    /// Trim whitespace from string with mode options
    StringTrim,
    // I/O
    ReadInput,
    FileRead,
    FileWrite,
    // Other
    InputParam,
    OutputParam,
    // Entry point for the graph
    Entry,
    // Type conversions
    ToInteger,
    ToFloat,
    ToString,
    // Timing
    Delay,
    /// Get current Unix timestamp
    GetTimestamp,
    // System Control
    RunCommand,
    LaunchApp,
    CloseApp,
    FocusWindow,
    GetWindowPosition,
    SetWindowPosition,
    // Data Operations (Module H)
    /// Create an empty array or array with initial values
    ArrayCreate,
    /// Push a value to the end of an array
    ArrayPush,
    /// Pop (remove and return) the last value from an array
    ArrayPop,
    /// Get a value at a specific index from an array
    ArrayGet,
    /// Set a value at a specific index in an array
    ArraySet,
    /// Get the length of an array
    ArrayLength,
    /// Parse a JSON string into a value
    JSONParse,
    /// Convert a value to a JSON string
    JSONStringify,
    /// Make an HTTP request (GET/POST)
    HTTPRequest,
    // Desktop Input Automation (Module A)
    /// Click at screen coordinates (x, y)
    Click,
    /// Double-click at coordinates
    DoubleClick,
    /// Right-click at coordinates
    RightClick,
    /// Move cursor to coordinates
    MouseMove,
    /// Press mouse button without releasing
    MouseDown,
    /// Release mouse button
    MouseUp,
    /// Mouse wheel scroll
    Scroll,
    /// Press and release a key
    KeyPress,
    /// Press key without releasing
    KeyDown,
    /// Release a pressed key
    KeyUp,
    /// Type a string of text
    TypeText,
    /// Type a string by simulating individual key presses with configurable delay
    TypeString,
    /// Key combinations (Ctrl+C, Cmd+V, etc.)
    HotKey,
    // Screenshot & Image Tools (Module C)
    /// Capture full screen or specific display
    ScreenCapture,
    /// Save screenshot to file
    SaveScreenshot,
    /// Capture a specific screen region to image file
    RegionCapture,
    // Image Recognition (Module D)
    /// Get RGB color at screen coordinates
    GetPixelColor,
    /// Search for a color in screen region
    FindColor,
    /// Wait until color appears at location
    WaitForColor,
    /// Template matching - find image on screen
    FindImage,
    /// Wait until image appears on screen
    WaitForImage,
    /// Compare two images with tolerance
    ImageSimilarity,
    /// Extract N characters after a keyword
    ExtractAfter,
    /// Extract content from keyword until delimiter
    ExtractUntil,
    // Advanced Control Flow
    /// Blocks execution until a condition becomes true
    WaitForCondition,
    /// For loop that waits for Continue signal before each iteration
    ForLoopAsync,
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Entry
    }
}
