use gtk::gdk;

const THEME_CSS: &str = r#"
window {
    background-color: #000000;
    color: #00FF00;
}

drawingarea {
    background-color: #000000;
}

entry {
    background-color: #1a1a1a;
    color: #00FF00;
    border: 1px solid #00FF00;
    padding: 10px;
    border-radius: 4px;
}

entry:focus {
    border-color: #00FF00;
    box-shadow: 0 0 5px rgba(0, 255, 0, 0.5);
}

button {
    background-color: #1a1a1a;
    color: #00FF00;
    border: 1px solid #00FF00;
    padding: 10px 20px;
    border-radius: 4px;
}

button:hover {
    background-color: #00FF00;
    color: #000000;
}

button.suggested-action {
    background-color: #00FF00;
    color: #000000;
    font-weight: bold;
}

button.suggested-action:hover {
    background-color: #00CC00;
}

button.link {
    background: none;
    border: none;
    color: #00FF00;
    text-decoration: underline;
}

label {
    color: #00FF00;
}

label.title-1 {
    font-size: 24px;
    font-weight: bold;
    margin-bottom: 10px;
}

label.dim-label {
    opacity: 0.7;
}

label.error {
    color: #FF0000;
    font-weight: bold;
}

.header {
    background-color: #0a0a0a;
    border-bottom: 2px solid #00FF00;
}

.title {
    font-size: 20px;
    font-weight: bold;
    color: #00FF00;
}

.price-indicator {
    font-size: 16px;
    font-weight: bold;
    color: #00FF00;
}

.panel {
    background-color: #0f0f0f;
    border: 1px solid #00FF00;
    border-radius: 4px;
}

.panel-title {
    font-size: 14px;
    font-weight: bold;
    color: #00FF00;
    margin: 10px;
}

.data-label {
    color: #00AA00;
    font-size: 12px;
    margin: 5px;
}

.footer {
    background-color: #0a0a0a;
    border-top: 1px solid #00FF00;
}

.price-symbol {
    font-size: 13px;
    font-weight: bold;
    color: #00FF00;
    font-family: monospace;
}

.price-value {
    font-size: 13px;
    color: #00CC00;
    font-family: monospace;
}

.window-control {
    min-width: 36px;
    min-height: 36px;
    padding: 6px;
    border-radius: 3px;
    background-color: transparent;
    border: none;
    font-size: 18px;
    font-weight: bold;
}

.window-control.minimize-btn {
    color: #00FF00;
}

.window-control.minimize-btn:hover {
    background-color: rgba(0, 255, 0, 0.2);
}

.window-control.close-btn {
    color: #00FF00;
}

.window-control.close-btn:hover {
    background-color: rgba(0, 255, 0, 0.3);
    color: #00FF00;
}
"#;

pub fn apply_theme() {
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(THEME_CSS);

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
