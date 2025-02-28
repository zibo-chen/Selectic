use crate::{Selection, SelectionError, Selector};
use arboard::Clipboard;
use enigo::{
    self,
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use log::{debug, error, info, warn};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;
use std::time::Duration;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationTextPattern, UIA_TextPatternId,
};

// 确保COM只初始化一次
static COM_INIT: Once = Once::new();
static COM_INIT_FAILED: AtomicBool = AtomicBool::new(false);

pub struct WindowsSelector {}

impl WindowsSelector {
    pub fn new() -> Self {
        // 在创建选择器时尝试初始化COM
        init_com();
        WindowsSelector {}
    }
}

impl Selector for WindowsSelector {
    fn get_selection(&self) -> Result<Selection, SelectionError> {
        get_windows_selection()
    }
}

fn init_com() {
    COM_INIT.call_once(|| {
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if hr.is_ok() {
            debug!("COM initialized successfully");
        } else {
            let error_code = hr.0;
            error!("Failed to initialize COM: HRESULT 0x{:08X}", error_code);
            COM_INIT_FAILED.store(true, Ordering::SeqCst);
        }
    });
}

fn get_windows_selection() -> Result<Selection, SelectionError> {
    debug!("Getting Windows selection...");

    let result = get_text_internal()?;

    if result.is_empty() {
        return Err(SelectionError::NoSelectedContent);
    }

    Ok(Selection::new_text(result))
}

fn get_text_internal() -> Result<String, SelectionError> {
    // 首先尝试UI自动化方法
    if !COM_INIT_FAILED.load(Ordering::SeqCst) {
        match get_text_by_automation() {
            Ok(text) if !text.is_empty() => {
                debug!(
                    "Successfully retrieved text via UI Automation: {} chars",
                    text.len()
                );
                return Ok(text);
            }
            Ok(_) => info!("UI Automation returned empty text"),
            Err(err) => error!("UI Automation error: {}", err),
        }
    } else {
        debug!("Skipping UI Automation due to COM initialization failure");
    }

    // 回退到剪贴板方法
    info!("Falling back to clipboard method");
    match get_text_by_clipboard() {
        Ok(text) if !text.is_empty() => {
            debug!(
                "Successfully retrieved text via clipboard: {} chars",
                text.len()
            );
            return Ok(text);
        }
        Ok(_) => info!("Clipboard method returned empty text"),
        Err(err) => {
            error!("Clipboard method error: {}", err);
            return Err(SelectionError::ClipboardError(err.to_string()));
        }
    }

    Err(SelectionError::NoSelectedContent)
}

fn get_text_by_automation() -> Result<String, Box<dyn Error>> {
    debug!("Attempting to get text via UI Automation");

    // 创建IUIAutomation实例
    let auto: IUIAutomation = unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL) }
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    // 获取焦点元素
    let el = unsafe { auto.GetFocusedElement() }.map_err(|e| {
        debug!("Failed to get focused element: {:?}", e);
        Box::new(SelectionError::NoFocusedElement) as Box<dyn Error>
    })?;

    // 尝试获取TextPattern
    let pattern_result =
        unsafe { el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) };

    let text_pattern = match pattern_result {
        Ok(pattern) => pattern,
        Err(e) => {
            debug!("No text pattern available: {:?}", e);
            return Ok(String::new());
        }
    };

    // 获取TextRange数组
    let text_array = unsafe { text_pattern.GetSelection() }.map_err(|e| {
        debug!("Failed to get selection: {:?}", e);
        Box::new(e) as Box<dyn Error>
    })?;

    let length = unsafe { text_array.Length() }.map_err(|e| {
        debug!("Failed to get text array length: {:?}", e);
        Box::new(e) as Box<dyn Error>
    })?;

    if length == 0 {
        debug!("No text ranges in selection");
        return Ok(String::new());
    }

    // 迭代TextRange数组
    let mut target = String::with_capacity(256); // 预分配合理的容量
    for i in 0..length {
        let text_range = unsafe { text_array.GetElement(i) }.map_err(|e| {
            debug!("Failed to get text range element {}: {:?}", i, e);
            Box::new(e) as Box<dyn Error>
        })?;

        // 指定合理的字符数量限制，-1表示获取所有
        let text = unsafe { text_range.GetText(1024) }.map_err(|e| {
            debug!("Failed to get text from range {}: {:?}", i, e);
            Box::new(e) as Box<dyn Error>
        })?;

        target.push_str(&text.to_string());
    }

    Ok(target.trim().to_string())
}

fn get_text_by_clipboard() -> Result<String, Box<dyn Error>> {
    debug!("Attempting to get text via clipboard");

    // 读取旧的剪贴板内容
    let mut clipboard = Clipboard::new().map_err(|e| {
        Box::new(SelectionError::ClipboardError(format!(
            "Failed to open clipboard: {}",
            e
        ))) as Box<dyn Error>
    })?;

    let old_text = clipboard.get_text().ok();
    let old_image = clipboard.get_image().ok();

    // 尝试复制选中内容到剪贴板
    if !copy() {
        return Err(Box::new(SelectionError::ClipboardError(
            "Copy operation failed".into(),
        )));
    }

    // 给系统一点时间处理剪贴板
    std::thread::sleep(Duration::from_millis(150));

    // 读取新的剪贴板内容
    let mut new_clipboard = Clipboard::new().map_err(|e| {
        Box::new(SelectionError::ClipboardError(format!(
            "Failed to open clipboard after copy: {}",
            e
        ))) as Box<dyn Error>
    })?;

    let new_text = new_clipboard.get_text().map_err(|e| {
        Box::new(SelectionError::ClipboardError(format!(
            "Failed to get text from clipboard: {}",
            e
        ))) as Box<dyn Error>
    })?;

    // 恢复原来的剪贴板内容
    restore_clipboard(old_text, old_image)?;

    // 返回新获取的文本
    if !new_text.is_empty() {
        Ok(new_text.trim().to_string())
    } else {
        Err(Box::new(SelectionError::NoSelectedContent))
    }
}

fn restore_clipboard(
    old_text: Option<String>,
    old_image: Option<arboard::ImageData>,
) -> Result<(), Box<dyn Error>> {
    let mut restore_clipboard = Clipboard::new().map_err(|e| {
        Box::new(SelectionError::ClipboardError(format!(
            "Failed to open clipboard for restoration: {}",
            e
        ))) as Box<dyn Error>
    })?;

    if let Some(text) = old_text {
        restore_clipboard.set_text(text).map_err(|e| {
            Box::new(SelectionError::ClipboardError(format!(
                "Failed to restore text to clipboard: {}",
                e
            ))) as Box<dyn Error>
        })?;
    } else if let Some(image) = old_image {
        restore_clipboard.set_image(image).map_err(|e| {
            Box::new(SelectionError::ClipboardError(format!(
                "Failed to restore image to clipboard: {}",
                e
            ))) as Box<dyn Error>
        })?;
    } else {
        restore_clipboard.clear().map_err(|e| {
            Box::new(SelectionError::ClipboardError(format!(
                "Failed to clear clipboard: {}",
                e
            ))) as Box<dyn Error>
        })?;
    }

    Ok(())
}

// 确保所有修饰键处于释放状态
fn release_keys() -> Result<(), Box<dyn Error>> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| {
        Box::new(SelectionError::Other(format!(
            "Failed to create Enigo instance: {}",
            e
        ))) as Box<dyn Error>
    })?;

    enigo.key(Key::Control, Release).map_err(|e| {
        Box::new(SelectionError::Other(format!(
            "Failed to release Control key: {}",
            e
        ))) as Box<dyn Error>
    })?;
    enigo.key(Key::Alt, Release).map_err(|e| {
        Box::new(SelectionError::Other(format!(
            "Failed to release Alt key: {}",
            e
        ))) as Box<dyn Error>
    })?;
    enigo.key(Key::Shift, Release).map_err(|e| {
        Box::new(SelectionError::Other(format!(
            "Failed to release Shift key: {}",
            e
        ))) as Box<dyn Error>
    })?;
    enigo.key(Key::Meta, Release).map_err(|e| {
        Box::new(SelectionError::Other(format!(
            "Failed to release Meta key: {}",
            e
        ))) as Box<dyn Error>
    })?;

    Ok(())
}

fn copy() -> bool {
    debug!("Executing copy command");

    // 记录复制前的剪贴板序列号
    let num_before = unsafe { GetClipboardSequenceNumber() };

    // 创建自动化引擎
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(e) => {
            error!("Failed to create Enigo instance: {:?}", e);
            return false;
        }
    };

    if let Err(e) = release_keys() {
        error!("Failed to release modifier keys: {:?}", e);
        return false;
    }

    // 执行Ctrl+C
    let copy_result = (|| -> Result<(), Box<dyn Error>> {
        enigo
            .key(Key::Control, Press)
            .map_err(|e| Box::new(e) as Box<dyn Error>)?;

        enigo.key(Key::C, Click).map_err(|e| {
            // 确保释放Ctrl键
            let _ = enigo.key(Key::Control, Release);
            Box::new(e) as Box<dyn Error>
        })?;

        enigo
            .key(Key::Control, Release)
            .map_err(|e| Box::new(e) as Box<dyn Error>)?;

        Ok(())
    })();

    if let Err(e) = copy_result {
        error!("Copy operation failed: {:?}", e);
        return false;
    }

    // 等待剪贴板更新
    std::thread::sleep(Duration::from_millis(150));

    // 检查剪贴板是否变化
    let num_after = unsafe { GetClipboardSequenceNumber() };
    let result = num_after != num_before;

    if !result {
        warn!("Clipboard sequence number did not change after copy attempt");
    }

    result
}
