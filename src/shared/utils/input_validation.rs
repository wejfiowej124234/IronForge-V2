//! Input Validation - 输入验证工具
//! 企业级输入验证和格式化工具

/// 验证金额输入
pub fn validate_amount(input: &str) -> Result<f64, String> {
    if input.is_empty() {
        return Err("请输入金额".to_string());
    }
    
    let amount = input.parse::<f64>()
        .map_err(|_| "请输入有效的数字".to_string())?;
    
    if amount <= 0.0 {
        return Err("金额必须大于0".to_string());
    }
    
    if amount > 1_000_000_000.0 {
        return Err("金额过大，请输入小于10亿的金额".to_string());
    }
    
    Ok(amount)
}

/// 格式化金额显示
pub fn format_amount(amount: f64, decimals: usize) -> String {
    if amount == 0.0 {
        return "0".to_string();
    }
    
    // 根据金额大小选择合适的小数位数
    let decimals = if amount >= 1000.0 {
        decimals.min(2)
    } else if amount >= 1.0 {
        decimals.min(4)
    } else {
        decimals.min(6)
    };
    
    format!("{:.decimals$}", amount, decimals = decimals)
}

/// 验证价格输入
pub fn validate_price(input: &str) -> Result<f64, String> {
    if input.is_empty() {
        return Err("请输入价格".to_string());
    }
    
    let price = input.parse::<f64>()
        .map_err(|_| "请输入有效的数字".to_string())?;
    
    if price <= 0.0 {
        return Err("价格必须大于0".to_string());
    }
    
    Ok(price)
}

/// 格式化价格显示
pub fn format_price(price: f64) -> String {
    if price == 0.0 {
        return "0".to_string();
    }
    
    if price >= 1000.0 {
        format!("{:.2}", price)
    } else if price >= 1.0 {
        format!("{:.4}", price)
    } else if price >= 0.01 {
        format!("{:.6}", price)
    } else {
        format!("{:.8}", price)
    }
}

/// 验证滑点输入
pub fn validate_slippage(input: &str) -> Result<f64, String> {
    if input.is_empty() {
        return Ok(0.5); // 默认滑点
    }
    
    let slippage = input.parse::<f64>()
        .map_err(|_| "请输入有效的数字".to_string())?;
    
    if slippage < 0.1 {
        return Err("滑点容忍度不能小于0.1%".to_string());
    }
    
    if slippage > 5.0 {
        return Err("滑点容忍度不能大于5%".to_string());
    }
    
    Ok(slippage)
}

/// 验证地址格式
pub fn validate_address(address: &str, chain_type: &str) -> Result<(), String> {
    if address.is_empty() {
        return Err("请输入地址".to_string());
    }
    
    match chain_type {
        "ethereum" | "bsc" | "polygon" => {
            if !address.starts_with("0x") || address.len() != 42 {
                return Err("请输入有效的以太坊地址（0x开头，42字符）".to_string());
            }
        }
        "bitcoin" => {
            if address.len() < 26 || address.len() > 62 {
                return Err("请输入有效的比特币地址".to_string());
            }
        }
        _ => {
            // 其他链的验证可以在这里添加
        }
    }
    
    Ok(())
}

/// 限制输入长度
pub fn limit_input_length(input: &str, max_length: usize) -> String {
    if input.len() > max_length {
        input.chars().take(max_length).collect()
    } else {
        input.to_string()
    }
}

/// 移除非法字符（只保留数字和小数点）
pub fn sanitize_numeric_input(input: &str) -> String {
    input.chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect()
}

/// 移除非法字符（只保留数字、字母和常见符号）
pub fn sanitize_address_input(input: &str) -> String {
    input.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == 'x' || *c == 'X' || *c == '0')
        .collect()
}

