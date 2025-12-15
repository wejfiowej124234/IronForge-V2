//! 翻译字典
//! 集中管理所有文本翻译

use std::collections::HashMap;
use std::sync::LazyLock;

/// 翻译字典类型
type TranslationDict = HashMap<&'static str, HashMap<&'static str, &'static str>>;

/// 全局翻译字典
static TRANSLATIONS: LazyLock<TranslationDict> = LazyLock::new(|| {
    let mut dict = HashMap::new();

    // ============ 通用词汇 ============
    add_translation(
        &mut dict,
        "common.back_to_dashboard",
        "zh",
        "返回仪表盘",
        "en",
        "Back to Dashboard",
        "ja",
        "ダッシュボードに戻る",
        "ko",
        "대시보드로 돌아가기",
    );

    add_translation(
        &mut dict,
        "common.login",
        "zh",
        "登录",
        "en",
        "Login",
        "ja",
        "ログイン",
        "ko",
        "로그인",
    );

    add_translation(
        &mut dict,
        "common.register",
        "zh",
        "注册",
        "en",
        "Register",
        "ja",
        "登録",
        "ko",
        "회원가입",
    );

    add_translation(
        &mut dict,
        "common.logout",
        "zh",
        "登出",
        "en",
        "Logout",
        "ja",
        "ログアウト",
        "ko",
        "로그아웃",
    );

    add_translation(
        &mut dict,
        "common.confirm",
        "zh",
        "确认",
        "en",
        "Confirm",
        "ja",
        "確認",
        "ko",
        "확인",
    );

    add_translation(
        &mut dict,
        "common.cancel",
        "zh",
        "取消",
        "en",
        "Cancel",
        "ja",
        "キャンセル",
        "ko",
        "취소",
    );

    // ============ 导航菜单 ============
    add_translation(
        &mut dict,
        "nav.dashboard",
        "zh",
        "仪表盘",
        "en",
        "Dashboard",
        "ja",
        "ダッシュボード",
        "ko",
        "대시보드",
    );

    add_translation(
        &mut dict,
        "nav.send",
        "zh",
        "发送",
        "en",
        "Send",
        "ja",
        "送信",
        "ko",
        "보내기",
    );

    add_translation(
        &mut dict,
        "nav.receive",
        "zh",
        "接收",
        "en",
        "Receive",
        "ja",
        "受信",
        "ko",
        "받기",
    );

    add_translation(
        &mut dict,
        "nav.swap",
        "zh",
        "交换",
        "en",
        "Swap",
        "ja",
        "スワップ",
        "ko",
        "교환",
    );

    // ============ 页面标题 ============
    add_translation(
        &mut dict,
        "page.swap.title",
        "zh",
        "🔄 代币交换",
        "en",
        "🔄 Token Swap",
        "ja",
        "🔄 トークンスワップ",
        "ko",
        "🔄 토큰 교환",
    );

    add_translation(
        &mut dict,
        "page.send.title",
        "zh",
        "💸 发送资产",
        "en",
        "💸 Send Assets",
        "ja",
        "💸 資産送信",
        "ko",
        "💸 자산 보내기",
    );

    add_translation(
        &mut dict,
        "page.receive.title",
        "zh",
        "💸 接收资产",
        "en",
        "💸 Receive Assets",
        "ja",
        "💸 資産受信",
        "ko",
        "💸 자산 받기",
    );

    add_translation(
        &mut dict,
        "page.withdraw.title",
        "zh",
        "💰 提现到银行卡",
        "en",
        "💰 Withdraw to Bank",
        "ja",
        "💰 銀行口座へ出金",
        "ko",
        "💰 은행으로 출금",
    );

    // ============ 钱包相关 ============
    add_translation(
        &mut dict,
        "wallet.balance",
        "zh",
        "余额",
        "en",
        "Balance",
        "ja",
        "残高",
        "ko",
        "잔액",
    );

    add_translation(
        &mut dict,
        "wallet.address",
        "zh",
        "钱包地址",
        "en",
        "Wallet Address",
        "ja",
        "ウォレットアドレス",
        "ko",
        "지갑 주소",
    );

    add_translation(
        &mut dict,
        "wallet.copy_address",
        "zh",
        "📋 复制地址",
        "en",
        "📋 Copy Address",
        "ja",
        "📋 アドレスをコピー",
        "ko",
        "📋 주소 복사",
    );

    add_translation(
        &mut dict,
        "wallet.copied",
        "zh",
        "✅ 已复制到剪贴板",
        "en",
        "✅ Copied to Clipboard",
        "ja",
        "✅ クリップボードにコピー済み",
        "ko",
        "✅ 클립보드에 복사됨",
    );

    // ============ 交易相关 ============
    add_translation(
        &mut dict,
        "transaction.amount",
        "zh",
        "金额",
        "en",
        "Amount",
        "ja",
        "金額",
        "ko",
        "금액",
    );

    add_translation(
        &mut dict,
        "transaction.fee",
        "zh",
        "手续费",
        "en",
        "Fee",
        "ja",
        "手数料",
        "ko",
        "수수료",
    );

    add_translation(
        &mut dict,
        "transaction.total",
        "zh",
        "总计",
        "en",
        "Total",
        "ja",
        "合計",
        "ko",
        "총액",
    );

    // ============ 安全提示 ============
    add_translation(
        &mut dict,
        "security.warning",
        "zh",
        "重要安全提示",
        "en",
        "Important Security Notice",
        "ja",
        "重要なセキュリティのお知らせ",
        "ko",
        "중요 보안 공지",
    );

    // ============ 表单标签 ============
    add_translation(
        &mut dict,
        "form.email",
        "zh",
        "邮箱",
        "en",
        "Email",
        "ja",
        "メール",
        "ko",
        "이메일",
    );
    add_translation(
        &mut dict,
        "form.password",
        "zh",
        "密码",
        "en",
        "Password",
        "ja",
        "パスワード",
        "ko",
        "비밀번호",
    );
    add_translation(
        &mut dict,
        "form.amount",
        "zh",
        "金额",
        "en",
        "Amount",
        "ja",
        "金額",
        "ko",
        "금액",
    );
    add_translation(
        &mut dict,
        "form.address",
        "zh",
        "地址",
        "en",
        "Address",
        "ja",
        "アドレス",
        "ko",
        "주소",
    );
    add_translation(
        &mut dict,
        "form.select_token",
        "zh",
        "选择代币",
        "en",
        "Select Token",
        "ja",
        "トークンを選択",
        "ko",
        "토큰 선택",
    );
    add_translation(
        &mut dict,
        "form.select_chain",
        "zh",
        "选择链",
        "en",
        "Select Chain",
        "ja",
        "チェーンを選択",
        "ko",
        "체인 선택",
    );

    // ============ 状态消息 ============
    add_translation(
        &mut dict,
        "status.loading",
        "zh",
        "加载中...",
        "en",
        "Loading...",
        "ja",
        "読み込み中...",
        "ko",
        "로딩 중...",
    );
    add_translation(
        &mut dict,
        "status.processing",
        "zh",
        "处理中...",
        "en",
        "Processing...",
        "ja",
        "処理中...",
        "ko",
        "처리 중...",
    );
    add_translation(
        &mut dict,
        "status.success",
        "zh",
        "成功",
        "en",
        "Success",
        "ja",
        "成功",
        "ko",
        "성공",
    );
    add_translation(
        &mut dict,
        "status.failed",
        "zh",
        "失败",
        "en",
        "Failed",
        "ja",
        "失敗",
        "ko",
        "실패",
    );

    // ============ 错误消息 ============
    add_translation(
        &mut dict,
        "error.required_field",
        "zh",
        "此字段为必填项",
        "en",
        "This field is required",
        "ja",
        "このフィールドは必須です",
        "ko",
        "이 필드는 필수입니다",
    );
    add_translation(
        &mut dict,
        "error.invalid_address",
        "zh",
        "地址无效",
        "en",
        "Invalid address",
        "ja",
        "無効なアドレス",
        "ko",
        "잘못된 주소",
    );
    add_translation(
        &mut dict,
        "error.insufficient_balance",
        "zh",
        "余额不足",
        "en",
        "Insufficient balance",
        "ja",
        "残高不足",
        "ko",
        "잔액 부족",
    );
    add_translation(
        &mut dict,
        "error.amount_too_large",
        "zh",
        "金额过大，请输入有效金额",
        "en",
        "Amount too large, please enter a valid amount",
        "ja",
        "金額が大きすぎます。有効な金額を入力してください",
        "ko",
        "금액이 너무 큽니다. 유효한 금액을 입력하세요",
    );
    add_translation(
        &mut dict,
        "error.same_token",
        "zh",
        "不能交换相同的代币",
        "en",
        "Cannot swap the same token",
        "ja",
        "同じトークンをスワップできません",
        "ko",
        "동일한 토큰을 스왑할 수 없습니다",
    );
    add_translation(
        &mut dict,
        "error.rate_limit",
        "zh",
        "请求过于频繁，请稍后再试",
        "en",
        "Too many requests, please try again later",
        "ja",
        "リクエストが多すぎます。後でもう一度お試しください",
        "ko",
        "요청이 너무 많습니다. 나중에 다시 시도하세요",
    );
    add_translation(
        &mut dict,
        "error.network_timeout",
        "zh",
        "请求超时，请检查网络连接后重试",
        "en",
        "Request timeout, please check your network and retry",
        "ja",
        "リクエストがタイムアウトしました。ネットワークを確認して再試行してください",
        "ko",
        "요청 시간 초과. 네트워크를 확인하고 다시 시도하세요",
    );
    add_translation(
        &mut dict,
        "error.network_failed",
        "zh",
        "网络连接失败，请检查网络连接",
        "en",
        "Network connection failed, please check your network",
        "ja",
        "ネットワーク接続に失敗しました。ネットワークを確認してください",
        "ko",
        "네트워크 연결 실패. 네트워크를 확인하세요",
    );
    add_translation(
        &mut dict,
        "error.service_unavailable",
        "zh",
        "服务暂时不可用，请稍后再试",
        "en",
        "Service temporarily unavailable, please try again later",
        "ja",
        "サービスは一時的に利用できません。後でもう一度お試しください",
        "ko",
        "서비스를 일시적으로 사용할 수 없습니다. 나중에 다시 시도하세요",
    );
    add_translation(
        &mut dict,
        "error.invalid_amount",
        "zh",
        "请输入有效的交换数量",
        "en",
        "Please enter a valid swap amount",
        "ja",
        "有効なスワップ数量を入力してください",
        "ko",
        "유효한 스왑 수량을 입력하세요",
    );
    add_translation(
        &mut dict,
        "error.select_from_token",
        "zh",
        "请选择支付代币",
        "en",
        "Please select payment token",
        "ja",
        "支払いトークンを選択してください",
        "ko",
        "지불 토큰을 선택하세요",
    );
    add_translation(
        &mut dict,
        "error.select_to_token",
        "zh",
        "请选择接收代币",
        "en",
        "Please select receiving token",
        "ja",
        "受取トークンを選択してください",
        "ko",
        "수신 토큰을 선택하세요",
    );
    add_translation(
        &mut dict,
        "error.get_quote_first",
        "zh",
        "请先获取报价",
        "en",
        "Please get a quote first",
        "ja",
        "最初に見積もりを取得してください",
        "ko",
        "먼저 견적을 받으세요",
    );
    add_translation(
        &mut dict,
        "error.select_wallet",
        "zh",
        "请先选择钱包",
        "en",
        "Please select a wallet first",
        "ja",
        "最初にウォレットを選択してください",
        "ko",
        "먼저 지갑을 선택하세요",
    );
    add_translation(
        &mut dict,
        "error.wallet_locked",
        "zh",
        "钱包未解锁，无法签名交易",
        "en",
        "Wallet locked, cannot sign transaction",
        "ja",
        "ウォレットがロックされており、取引に署名できません",
        "ko",
        "지갑이 잠겨 있어 거래에 서명할 수 없습니다",
    );
    add_translation(
        &mut dict,
        "error.account_not_found",
        "zh",
        "钱包账户不存在",
        "en",
        "Wallet account not found",
        "ja",
        "ウォレットアカウントが見つかりません",
        "ko",
        "지갑 계정을 찾을 수 없습니다",
    );

    // ============ 仪表盘 ============
    add_translation(
        &mut dict,
        "dashboard.total_balance",
        "zh",
        "总资产",
        "en",
        "Total Balance",
        "ja",
        "総資産",
        "ko",
        "총 자산",
    );
    add_translation(
        &mut dict,
        "dashboard.my_wallets",
        "zh",
        "我的钱包",
        "en",
        "My Wallets",
        "ja",
        "マイウォレット",
        "ko",
        "내 지갑",
    );
    add_translation(
        &mut dict,
        "dashboard.create_wallet",
        "zh",
        "创建钱包",
        "en",
        "Create Wallet",
        "ja",
        "ウォレット作成",
        "ko",
        "지갑 생성",
    );
    add_translation(
        &mut dict,
        "dashboard.quick_actions",
        "zh",
        "快速操作",
        "en",
        "Quick Actions",
        "ja",
        "クイックアクション",
        "ko",
        "빠른 작업",
    );

    // ============ 发送页面 ============
    add_translation(
        &mut dict,
        "send.recipient",
        "zh",
        "接收地址",
        "en",
        "Recipient Address",
        "ja",
        "受取アドレス",
        "ko",
        "수신 주소",
    );
    add_translation(
        &mut dict,
        "send.confirm_transaction",
        "zh",
        "确认交易",
        "en",
        "Confirm Transaction",
        "ja",
        "取引を確認",
        "ko",
        "거래 확인",
    );

    // ============ 接收页面 ============
    add_translation(
        &mut dict,
        "receive.scan_qr",
        "zh",
        "扫描二维码",
        "en",
        "Scan QR Code",
        "ja",
        "QRコードをスキャン",
        "ko",
        "QR 코드 스캔",
    );
    add_translation(
        &mut dict,
        "receive.share_address",
        "zh",
        "分享地址",
        "en",
        "Share Address",
        "ja",
        "アドレスを共有",
        "ko",
        "주소 공유",
    );

    // ============ 提现/出售页面 ============
    add_translation(
        &mut dict,
        "page.withdraw.title",
        "zh",
        "提现到银行卡",
        "en",
        "Withdraw to Bank",
        "ja",
        "銀行へ出金",
        "ko",
        "은행으로 출금",
    );
    add_translation(
        &mut dict,
        "withdraw.method",
        "zh",
        "提现方式",
        "en",
        "Withdrawal Method",
        "ja",
        "出金方法",
        "ko",
        "출금 방법",
    );
    add_translation(
        &mut dict,
        "withdraw.bank_card",
        "zh",
        "银行卡/借记卡",
        "en",
        "Bank Card/Debit Card",
        "ja",
        "銀行カード/デビットカード",
        "ko",
        "은행 카드/직불 카드",
    );
    add_translation(
        &mut dict,
        "withdraw.alipay",
        "zh",
        "支付宝 Alipay",
        "en",
        "Alipay",
        "ja",
        "アリペイ",
        "ko",
        "알리페이",
    );
    add_translation(
        &mut dict,
        "withdraw.wechat_pay",
        "zh",
        "微信支付 WeChat Pay",
        "en",
        "WeChat Pay",
        "ja",
        "ウィーチャットペイ",
        "ko",
        "위챗페이",
    );
    add_translation(
        &mut dict,
        "withdraw.currency",
        "zh",
        "法币币种",
        "en",
        "Fiat Currency",
        "ja",
        "法定通貨",
        "ko",
        "법정 화폐",
    );
    add_translation(
        &mut dict,
        "withdraw.need_login",
        "zh",
        "需要登录",
        "en",
        "Login Required",
        "ja",
        "ログインが必要です",
        "ko",
        "로그인 필요",
    );
    add_translation(
        &mut dict,
        "withdraw.login_prompt",
        "zh",
        "请先登录您的账户，然后再进行法币提现操作。",
        "en",
        "Please log in to your account before withdrawing to fiat.",
        "ja",
        "法定通貨への出金を行う前に、アカウントにログインしてください。",
        "ko",
        "법정 화폐 출금 전에 계정에 로그인하세요.",
    );
    add_translation(
        &mut dict,
        "withdraw.step1_select",
        "zh",
        "选择代币",
        "en",
        "Select Token",
        "ja",
        "選択",
        "ko",
        "선택",
    );
    add_translation(
        &mut dict,
        "withdraw.step2_method",
        "zh",
        "选择方式",
        "en",
        "Select Method",
        "ja",
        "方法",
        "ko",
        "방법",
    );
    add_translation(
        &mut dict,
        "withdraw.step3_info",
        "zh",
        "收款信息",
        "en",
        "Payment Info",
        "ja",
        "情報",
        "ko",
        "정보",
    );
    add_translation(
        &mut dict,
        "withdraw.step4_confirm",
        "zh",
        "确认提现",
        "en",
        "Confirm",
        "ja",
        "確認",
        "ko",
        "확인",
    );
    add_translation(&mut dict, "withdraw.two_step_hint",
        "zh", "系统将自动执行两步流程：代币 → 稳定币 → 法币。您只需选择要提现的代币和金额即可。",
        "en", "System will auto-execute: Token → Stablecoin → Fiat. Just select token and amount.",
        "ja", "システムが自動実行：トークン → ステーブルコイン → 法定通貨。トークンと金額を選択するだけです。",
        "ko", "시스템이 자동 실행: 토큰 → 스테이블코인 → 법정화폐. 토큰과 금액만 선택하세요."
    );
    add_translation(
        &mut dict,
        "withdraw.amount_label",
        "zh",
        "提现数量",
        "en",
        "Withdraw Amount",
        "ja",
        "出金数量",
        "ko",
        "출금 수량",
    );
    add_translation(
        &mut dict,
        "withdraw.select_token",
        "zh",
        "提现代币",
        "en",
        "Withdraw Token",
        "ja",
        "出金トークン",
        "ko",
        "출금 토큰",
    );

    // ============ 交换页面扩展 ============
    add_translation(
        &mut dict,
        "swap.buy_stablecoin",
        "zh",
        "购买稳定币",
        "en",
        "Buy Stablecoin",
        "ja",
        "ステーブルコイン購入",
        "ko",
        "스테이블코인 구매",
    );
    add_translation(
        &mut dict,
        "swap.limit_order",
        "zh",
        "限价单",
        "en",
        "Limit Order",
        "ja",
        "指値注文",
        "ko",
        "지정가 주문",
    );
    add_translation(
        &mut dict,
        "swap.history",
        "zh",
        "历史",
        "en",
        "History",
        "ja",
        "履歴",
        "ko",
        "기록",
    );
    add_translation(
        &mut dict,
        "swap.token_exchange",
        "zh",
        "代币交换",
        "en",
        "Token Exchange",
        "ja",
        "トークン交換",
        "ko",
        "토큰 교환",
    );
    add_translation(&mut dict, "swap.select_wallet_prompt",
        "zh", "请先在仪表盘选择并解锁一个钱包，然后再进行交换、买入或提现操作。",
        "en", "Please select and unlock a wallet in the dashboard before swapping, buying, or withdrawing.",
        "ja", "スワップ、購入、出金を行う前に、ダッシュボードでウォレットを選択してロックを解除してください。",
        "ko", "스왑, 구매 또는 출금 전에 대시보드에서 지갑을 선택하고 잠금 해제하세요."
    );
    add_translation(
        &mut dict,
        "swap.go_to_dashboard",
        "zh",
        "前往仪表盘选择钱包",
        "en",
        "Go to Dashboard to Select Wallet",
        "ja",
        "ダッシュボードへ移動してウォレットを選択",
        "ko",
        "대시보드로 이동하여 지갑 선택",
    );
    add_translation(
        &mut dict,
        "swap.beginner_guide",
        "zh",
        "新手引导",
        "en",
        "Beginner's Guide",
        "ja",
        "初心者ガイド",
        "ko",
        "초보자 가이드",
    );

    // ============ 登录/注册页面 ============
    add_translation(
        &mut dict,
        "page.login.title",
        "zh",
        "登录账户",
        "en",
        "Login to Account",
        "ja",
        "アカウントにログイン",
        "ko",
        "계정 로그인",
    );
    add_translation(
        &mut dict,
        "page.register.title",
        "zh",
        "注册账户",
        "en",
        "Register Account",
        "ja",
        "アカウント登録",
        "ko",
        "계정 등록",
    );
    add_translation(
        &mut dict,
        "login.subtitle",
        "zh",
        "登录您的 IronForge 账户",
        "en",
        "Login to your IronForge account",
        "ja",
        "IronForge アカウントにログイン",
        "ko",
        "IronForge 계정에 로그인",
    );
    add_translation(
        &mut dict,
        "login.go_to_login",
        "zh",
        "前往登录",
        "en",
        "Go to Login",
        "ja",
        "ログインへ移動",
        "ko",
        "로그인으로 이동",
    );
    add_translation(
        &mut dict,
        "login.no_account",
        "zh",
        "还没有账户？",
        "en",
        "Don't have an account?",
        "ja",
        "アカウントをお持ちでないですか？",
        "ko",
        "계정이 없으신가요?",
    );
    add_translation(
        &mut dict,
        "login.register_now",
        "zh",
        "立即注册",
        "en",
        "Register Now",
        "ja",
        "今すぐ登録",
        "ko",
        "지금 등록",
    );
    add_translation(
        &mut dict,
        "login.success",
        "zh",
        "登录成功",
        "en",
        "Login Successful",
        "ja",
        "ログイン成功",
        "ko",
        "로그인 성공",
    );
    add_translation(
        &mut dict,
        "login.failed",
        "zh",
        "登录失败",
        "en",
        "Login Failed",
        "ja",
        "ログイン失敗",
        "ko",
        "로그인 실패",
    );

    // ============ 钱包管理 ============
    add_translation(
        &mut dict,
        "wallet.create",
        "zh",
        "创建钱包",
        "en",
        "Create Wallet",
        "ja",
        "ウォレット作成",
        "ko",
        "지갑 생성",
    );
    add_translation(
        &mut dict,
        "wallet.import",
        "zh",
        "导入/恢复钱包",
        "en",
        "Import/Restore Wallet",
        "ja",
        "ウォレットをインポート/復元",
        "ko",
        "지갑 가져오기/복원",
    );
    add_translation(
        &mut dict,
        "wallet.name",
        "zh",
        "钱包名称",
        "en",
        "Wallet Name",
        "ja",
        "ウォレット名",
        "ko",
        "지갑 이름",
    );
    add_translation(
        &mut dict,
        "wallet.password",
        "zh",
        "密码",
        "en",
        "Password",
        "ja",
        "パスワード",
        "ko",
        "비밀번호",
    );
    add_translation(
        &mut dict,
        "wallet.confirm_password",
        "zh",
        "确认密码",
        "en",
        "Confirm Password",
        "ja",
        "パスワード確認",
        "ko",
        "비밀번호 확인",
    );
    add_translation(
        &mut dict,
        "wallet.enter_name",
        "zh",
        "请输入钱包名称",
        "en",
        "Please enter wallet name",
        "ja",
        "ウォレット名を入力してください",
        "ko",
        "지갑 이름을 입력하세요",
    );
    add_translation(
        &mut dict,
        "wallet.enter_password",
        "zh",
        "请输入密码",
        "en",
        "Please enter password",
        "ja",
        "パスワードを入力してください",
        "ko",
        "비밀번호를 입력하세요",
    );
    add_translation(
        &mut dict,
        "wallet.enter_password_again",
        "zh",
        "请再次输入密码",
        "en",
        "Please enter password again",
        "ja",
        "パスワードを再入力してください",
        "ko",
        "비밀번호를 다시 입력하세요",
    );
    add_translation(
        &mut dict,
        "wallet.name_required",
        "zh",
        "钱包名称不能为空",
        "en",
        "Wallet name cannot be empty",
        "ja",
        "ウォレット名は空欄にできません",
        "ko",
        "지갑 이름은 비워둘 수 없습니다",
    );
    add_translation(
        &mut dict,
        "wallet.password_min_length",
        "zh",
        "密码至少需要8个字符",
        "en",
        "Password must be at least 8 characters",
        "ja",
        "パスワードは8文字以上である必要があります",
        "ko",
        "비밀번호는 최소 8자 이상이어야 합니다",
    );
    add_translation(
        &mut dict,
        "wallet.password_mismatch",
        "zh",
        "两次输入的密码不一致",
        "en",
        "Passwords do not match",
        "ja",
        "パスワードが一致しません",
        "ko",
        "비밀번호가 일치하지 않습니다",
    );
    add_translation(
        &mut dict,
        "wallet.created_success",
        "zh",
        "钱包创建成功，请备份助记词",
        "en",
        "Wallet created successfully, please backup your mnemonic",
        "ja",
        "ウォレットが正常に作成されました。ニーモニックをバックアップしてください",
        "ko",
        "지갑이 성공적으로 생성되었습니다. 니모닉을 백업하세요",
    );
    add_translation(
        &mut dict,
        "wallet.create_failed",
        "zh",
        "创建钱包失败",
        "en",
        "Failed to create wallet",
        "ja",
        "ウォレットの作成に失敗しました",
        "ko",
        "지갑 생성 실패",
    );
    add_translation(
        &mut dict,
        "wallet.locked",
        "zh",
        "已锁定",
        "en",
        "Locked",
        "ja",
        "ロック済み",
        "ko",
        "잠김",
    );
    add_translation(
        &mut dict,
        "wallet.unlocked",
        "zh",
        "已解锁",
        "en",
        "Unlocked",
        "ja",
        "ロック解除済み",
        "ko",
        "잠금 해제됨",
    );
    add_translation(
        &mut dict,
        "wallet.status",
        "zh",
        "状态",
        "en",
        "Status",
        "ja",
        "ステータス",
        "ko",
        "상태",
    );
    add_translation(
        &mut dict,
        "wallet.id",
        "zh",
        "钱包ID",
        "en",
        "Wallet ID",
        "ja",
        "ウォレットID",
        "ko",
        "지갑 ID",
    );
    add_translation(
        &mut dict,
        "wallet.accounts",
        "zh",
        "账户数量",
        "en",
        "Number of Accounts",
        "ja",
        "アカウント数",
        "ko",
        "계정 수",
    );
    add_translation(
        &mut dict,
        "wallet.created_time",
        "zh",
        "创建时间",
        "en",
        "Created Time",
        "ja",
        "作成時刻",
        "ko",
        "생성 시간",
    );
    add_translation(
        &mut dict,
        "wallet.account_list",
        "zh",
        "账户列表",
        "en",
        "Account List",
        "ja",
        "アカウント一覧",
        "ko",
        "계정 목록",
    );
    add_translation(
        &mut dict,
        "wallet.not_found",
        "zh",
        "钱包未找到",
        "en",
        "Wallet Not Found",
        "ja",
        "ウォレットが見つかりません",
        "ko",
        "지갑을 찾을 수 없습니다",
    );
    add_translation(
        &mut dict,
        "wallet.details",
        "zh",
        "钱包详情",
        "en",
        "Wallet Details",
        "ja",
        "ウォレット詳細",
        "ko",
        "지갑 세부정보",
    );

    // ============ 交易历史 ============
    add_translation(
        &mut dict,
        "transaction.history",
        "zh",
        "交易历史",
        "en",
        "Transaction History",
        "ja",
        "取引履歴",
        "ko",
        "거래 내역",
    );
    add_translation(
        &mut dict,
        "transaction.loading",
        "zh",
        "正在加载交易历史...",
        "en",
        "Loading transaction history...",
        "ja",
        "取引履歴を読み込み中...",
        "ko",
        "거래 내역 로드 중...",
    );
    add_translation(
        &mut dict,
        "transaction.no_records",
        "zh",
        "暂无交易记录",
        "en",
        "No transaction records",
        "ja",
        "取引記録がありません",
        "ko",
        "거래 기록 없음",
    );
    add_translation(
        &mut dict,
        "transaction.error",
        "zh",
        "错误信息",
        "en",
        "Error Message",
        "ja",
        "エラーメッセージ",
        "ko",
        "오류 메시지",
    );

    // ============ 通用UI ============
    add_translation(
        &mut dict,
        "common.loading",
        "zh",
        "加载中...",
        "en",
        "Loading...",
        "ja",
        "読み込み中...",
        "ko",
        "로딩 중...",
    );
    add_translation(
        &mut dict,
        "common.return",
        "zh",
        "返回",
        "en",
        "Return",
        "ja",
        "戻る",
        "ko",
        "돌아가기",
    );
    add_translation(
        &mut dict,
        "common.return_dashboard",
        "zh",
        "返回Dashboard",
        "en",
        "Return to Dashboard",
        "ja",
        "ダッシュボードに戻る",
        "ko",
        "대시보드로 돌아가기",
    );
    add_translation(
        &mut dict,
        "common.email_placeholder",
        "zh",
        "请输入邮箱地址",
        "en",
        "Please enter email address",
        "ja",
        "メールアドレスを入力してください",
        "ko",
        "이메일 주소를 입력하세요",
    );
    add_translation(
        &mut dict,
        "common.password_placeholder",
        "zh",
        "请输入密码",
        "en",
        "Please enter password",
        "ja",
        "パスワードを入力してください",
        "ko",
        "비밀번호를 입력하세요",
    );
    add_translation(
        &mut dict,
        "common.email_invalid",
        "zh",
        "请输入有效的邮箱地址",
        "en",
        "Please enter a valid email address",
        "ja",
        "有効なメールアドレスを入力してください",
        "ko",
        "유효한 이메일 주소를 입력하세요",
    );
    add_translation(
        &mut dict,
        "common.password_required",
        "zh",
        "请输入密码",
        "en",
        "Please enter password",
        "ja",
        "パスワードを入力してください",
        "ko",
        "비밀번호를 입력하세요",
    );

    // ============ 提示信息 ============
    add_translation(
        &mut dict,
        "tip.create_wallet",
        "zh",
        "创建钱包：生成新钱包和助记词",
        "en",
        "Create Wallet: Generate new wallet and mnemonic",
        "ja",
        "ウォレット作成：新しいウォレットとニーモニックを生成",
        "ko",
        "지갑 생성: 새 지갑 및 니모닉 생성",
    );
    add_translation(
        &mut dict,
        "tip.import_wallet",
        "zh",
        "导入/恢复钱包：使用助记词或私钥在新设备上恢复钱包",
        "en",
        "Import/Restore: Restore wallet using mnemonic or private key on new device",
        "ja",
        "インポート/復元：ニーモニックまたは秘密鍵を使用して新しいデバイスでウォレットを復元",
        "ko",
        "가져오기/복원: 새 기기에서 니모닉 또는 개인 키를 사용하여 지갑 복원",
    );
    add_translation(
        &mut dict,
        "tip.label",
        "zh",
        "💡 提示：",
        "en",
        "💡 Tip:",
        "ja",
        "💡 ヒント：",
        "ko",
        "💡 팁:",
    );

    // ============ Swap 表单 ============
    add_translation(
        &mut dict,
        "swap.from_label",
        "zh",
        "支付",
        "en",
        "From",
        "ja",
        "支払い",
        "ko",
        "지불",
    );
    add_translation(
        &mut dict,
        "swap.to_label",
        "zh",
        "接收",
        "en",
        "To",
        "ja",
        "受取",
        "ko",
        "받기",
    );
    add_translation(
        &mut dict,
        "swap.select_token_placeholder",
        "zh",
        "选择代币",
        "en",
        "Select Token",
        "ja",
        "トークンを選択",
        "ko",
        "토큰 선택",
    );
    add_translation(
        &mut dict,
        "swap.amount_label",
        "zh",
        "数量",
        "en",
        "Amount",
        "ja",
        "数量",
        "ko",
        "수량",
    );
    add_translation(
        &mut dict,
        "swap.slippage_label",
        "zh",
        "最大滑点容差",
        "en",
        "Max Slippage",
        "ja",
        "最大スリッページ",
        "ko",
        "최대 슬리피지",
    );
    add_translation(
        &mut dict,
        "swap.max_button",
        "zh",
        "最大",
        "en",
        "Max",
        "ja",
        "最大",
        "ko",
        "최대",
    );
    add_translation(
        &mut dict,
        "swap.balance_insufficient",
        "zh",
        "稳定币余额不足",
        "en",
        "Insufficient Stablecoin Balance",
        "ja",
        "ステーブルコイン残高不足",
        "ko",
        "스테이블코인 잔액 부족",
    );
    add_translation(&mut dict, "swap.balance_warning",
        "zh", "您的稳定币余额不足，请先购买或充值 USDT/USDC 后再进行代币交换。",
        "en", "Your stablecoin balance is insufficient. Please buy or top up USDT/USDC before swapping.",
        "ja", "ステーブルコイン残高が不足しています。スワップ前にUSDT/USDCを購入またはチャージしてください。",
        "ko", "스테이블코인 잔액이 부족합니다. 스왑 전에 USDT/USDC를 구매하거나 충전하세요."
    );
    add_translation(
        &mut dict,
        "swap.go_buy_stablecoin",
        "zh",
        "立即购买稳定币",
        "en",
        "Buy Stablecoin Now",
        "ja",
        "今すぐ購入",
        "ko",
        "지금 구매",
    );
    add_translation(
        &mut dict,
        "swap.execute_button",
        "zh",
        "执行交换",
        "en",
        "Execute Swap",
        "ja",
        "スワップ実行",
        "ko",
        "스왑 실행",
    );
    add_translation(
        &mut dict,
        "swap.executing",
        "zh",
        "执行中...",
        "en",
        "Executing...",
        "ja",
        "実行中...",
        "ko",
        "실행 중...",
    );
    add_translation(
        &mut dict,
        "swap.rate",
        "zh",
        "汇率",
        "en",
        "Rate",
        "ja",
        "レート",
        "ko",
        "환율",
    );
    add_translation(
        &mut dict,
        "swap.estimated_receive",
        "zh",
        "预计收到",
        "en",
        "Estimated Receive",
        "ja",
        "予想受取",
        "ko",
        "예상 수령",
    );
    add_translation(
        &mut dict,
        "swap.slippage",
        "zh",
        "滑点",
        "en",
        "Slippage",
        "ja",
        "スリッページ",
        "ko",
        "슬리피지",
    );
    add_translation(
        &mut dict,
        "swap.price_trend_24h",
        "zh",
        "价格走势（24小时）",
        "en",
        "Price Trend (24h)",
        "ja",
        "価格推移（24時間）",
        "ko",
        "가격 추세 (24시간)",
    );
    add_translation(
        &mut dict,
        "swap.two_step_flow",
        "zh",
        "自动两步流程",
        "en",
        "Auto Two-Step Process",
        "ja",
        "自動2段階プロセス",
        "ko",
        "자동 2단계 프로세스",
    );
    add_translation(
        &mut dict,
        "swap.two_step_desc",
        "zh",
        "系统将自动执行：{} → 稳定币 → {}，您无需额外操作。",
        "en",
        "System will auto-execute: {} → Stablecoin → {}, no extra steps needed.",
        "ja",
        "システムが自動実行：{} → ステーブルコイン → {}、追加操作不要。",
        "ko",
        "시스템이 자동 실행: {} → 스테이블코인 → {}、추가 작업 불필요。",
    );

    // ============ Token Selector ============
    add_translation(
        &mut dict,
        "token.search_placeholder",
        "zh",
        "搜索钱包中的代币...",
        "en",
        "Search tokens in wallet...",
        "ja",
        "ウォレット内のトークンを検索...",
        "ko",
        "지갑에서 토큰 검색...",
    );
    add_translation(
        &mut dict,
        "token.only_show_balance",
        "zh",
        "只显示有余额的代币 · 共 1 个",
        "en",
        "Only show tokens with balance · 1 total",
        "ja",
        "残高のあるトークンのみ表示 · 合計1個",
        "ko",
        "잔액이 있는 토큰만 표시 · 총 1개",
    );
    add_translation(
        &mut dict,
        "token.ethereum_native",
        "zh",
        "Ethereum Native Token",
        "en",
        "Ethereum Native Token",
        "ja",
        "Ethereum ネイティブトークン",
        "ko",
        "이더리움 네이티브 토큰",
    );

    // ============ Buy Stablecoin (购买稳定币) ============
    add_translation(
        &mut dict,
        "buy.title",
        "zh",
        "购买稳定币",
        "en",
        "Buy Stablecoin",
        "ja",
        "ステーブルコイン購入",
        "ko",
        "스테이블코인 구매",
    );
    add_translation(
        &mut dict,
        "buy.kyc_required",
        "zh",
        "需要完成KYC验证",
        "en",
        "KYC Verification Required",
        "ja",
        "KYC認証が必要",
        "ko",
        "KYC 인증 필요",
    );
    add_translation(&mut dict, "buy.kyc_description",
        "zh", "为了满足全球安全合规要求，请完成KYC验证。完成验证后，您将获得更高安全交易额度。",
        "en", "To comply with global security regulations, please complete KYC verification. After verification, you will receive higher transaction limits.",
        "ja", "グローバルセキュリティ規制に準拠するため、KYC認証を完了してください。認証完了後、より高い取引限度額を取得できます。",
        "ko", "글로벌 보안 규정을 준수하기 위해 KYC 인증을 완료하세요. 인증 후 더 높은 거래 한도를 받을 수 있습니다."
    );
    add_translation(
        &mut dict,
        "buy.complete_kyc",
        "zh",
        "请先通过KYC验证",
        "en",
        "Please Complete KYC First",
        "ja",
        "まずKYC認証を完了してください",
        "ko",
        "먼저 KYC 인증을 완료하세요",
    );
    add_translation(
        &mut dict,
        "buy.step_select_token",
        "zh",
        "选择稳定币",
        "en",
        "Select Stablecoin",
        "ja",
        "ステーブルコイン選択",
        "ko",
        "스테이블코인 선택",
    );
    add_translation(
        &mut dict,
        "buy.step_enter_amount",
        "zh",
        "输入金额",
        "en",
        "Enter Amount",
        "ja",
        "金額入力",
        "ko",
        "금액 입력",
    );
    add_translation(
        &mut dict,
        "buy.step_select_payment",
        "zh",
        "查看价格",
        "en",
        "Check Price",
        "ja",
        "価格確認",
        "ko",
        "가격 확인",
    );
    add_translation(
        &mut dict,
        "buy.step_confirm",
        "zh",
        "确认购买",
        "en",
        "Confirm Purchase",
        "ja",
        "購入確認",
        "ko",
        "구매 확인",
    );
    add_translation(
        &mut dict,
        "buy.step1_select",
        "zh",
        "选择稳定币",
        "en",
        "Select Stablecoin",
        "ja",
        "選択",
        "ko",
        "선택",
    );
    add_translation(
        &mut dict,
        "buy.step2_amount",
        "zh",
        "输入金额",
        "en",
        "Enter Amount",
        "ja",
        "金額",
        "ko",
        "금액",
    );
    add_translation(
        &mut dict,
        "buy.step3_quote",
        "zh",
        "查看报价",
        "en",
        "View Quote",
        "ja",
        "見積",
        "ko",
        "견적",
    );
    add_translation(
        &mut dict,
        "buy.step4_confirm",
        "zh",
        "确认购买",
        "en",
        "Confirm",
        "ja",
        "確認",
        "ko",
        "확인",
    );
    add_translation(
        &mut dict,
        "buy.select_stablecoin",
        "zh",
        "购买稳定币",
        "en",
        "Buy Stablecoin",
        "ja",
        "ステーブルコイン購入",
        "ko",
        "스테이블코인 구매",
    );
    add_translation(
        &mut dict,
        "buy.choose_stablecoin",
        "zh",
        "选择稳定币",
        "en",
        "Choose Stablecoin",
        "ja",
        "ステーブルコイン選択",
        "ko",
        "스테이블코인 선택",
    );
    add_translation(
        &mut dict,
        "buy.purchase_amount",
        "zh",
        "购买金额",
        "en",
        "Purchase Amount",
        "ja",
        "購入金額",
        "ko",
        "구매 금액",
    );
    add_translation(
        &mut dict,
        "buy.enter_amount_placeholder",
        "zh",
        "输入金额（最小$10）",
        "en",
        "Enter amount (min $10)",
        "ja",
        "金額を入力（最小$10）",
        "ko",
        "금액 입력（최소 $10）",
    );
    add_translation(
        &mut dict,
        "buy.payment_method",
        "zh",
        "支付方式",
        "en",
        "Payment Method",
        "ja",
        "支払い方法",
        "ko",
        "결제 방법",
    );
    add_translation(
        &mut dict,
        "buy.bank_card",
        "zh",
        "信用卡/借记卡",
        "en",
        "Credit/Debit Card",
        "ja",
        "クレジット/デビットカード",
        "ko",
        "신용/직불 카드",
    );
    add_translation(
        &mut dict,
        "buy.bank_instant",
        "zh",
        "即时到账 · 支持Visa/Mastercard",
        "en",
        "Instant · Visa/Mastercard",
        "ja",
        "即時 · Visa/Mastercard",
        "ko",
        "즉시 · Visa/Mastercard",
    );
    add_translation(
        &mut dict,
        "buy.paypal_instant",
        "zh",
        "即时到账 · 全球支付",
        "en",
        "Instant · Global Payment",
        "ja",
        "即時 · グローバル決済",
        "ko",
        "즉시 · 글로벌 결제",
    );
    add_translation(
        &mut dict,
        "buy.apple_pay_instant",
        "zh",
        "即时到账 · iOS设备",
        "en",
        "Instant · iOS Device",
        "ja",
        "即時 · iOSデバイス",
        "ko",
        "즉시 · iOS 기기",
    );
    add_translation(
        &mut dict,
        "buy.google_pay_instant",
        "zh",
        "即时到账 · Android设备",
        "en",
        "Instant · Android Device",
        "ja",
        "即時 · Androidデバイス",
        "ko",
        "즉시 · 안드로이드 기기",
    );
    add_translation(
        &mut dict,
        "buy.alipay_instant",
        "zh",
        "即时到账 · 中国地区",
        "en",
        "Instant · China Region",
        "ja",
        "即時 · 中国地域",
        "ko",
        "즉시 · 중국 지역",
    );
    add_translation(
        &mut dict,
        "buy.wechat_instant",
        "zh",
        "即时到账 · 中国地区",
        "en",
        "Instant · China Region",
        "ja",
        "即時 · 中国地域",
        "ko",
        "즉시 · 중국 지역",
    );
    add_translation(
        &mut dict,
        "buy.button",
        "zh",
        "购买 USDT",
        "en",
        "Buy USDT",
        "ja",
        "USDT購入",
        "ko",
        "USDT 구매",
    );

    // ============ Withdraw (提现) ============
    add_translation(
        &mut dict,
        "withdraw.step_select",
        "zh",
        "选择代币",
        "en",
        "Select Token",
        "ja",
        "トークン選択",
        "ko",
        "토큰 선택",
    );
    add_translation(
        &mut dict,
        "withdraw.step_method",
        "zh",
        "选择方式",
        "en",
        "Select Method",
        "ja",
        "方法選択",
        "ko",
        "방법 선택",
    );
    add_translation(
        &mut dict,
        "withdraw.step_info",
        "zh",
        "收款信息",
        "en",
        "Payment Info",
        "ja",
        "受取情報",
        "ko",
        "수령 정보",
    );
    add_translation(
        &mut dict,
        "withdraw.step_confirm",
        "zh",
        "确认提现",
        "en",
        "Confirm",
        "ja",
        "確認",
        "ko",
        "확인",
    );
    add_translation(
        &mut dict,
        "withdraw.amount",
        "zh",
        "提现数量",
        "en",
        "Withdrawal Amount",
        "ja",
        "出金数量",
        "ko",
        "출금 수량",
    );

    dict
});

/// 辅助函数：添加多语言翻译
#[allow(clippy::too_many_arguments)]
fn add_translation(
    dict: &mut TranslationDict,
    key: &'static str,
    zh_key: &'static str,
    zh_val: &'static str,
    en_key: &'static str,
    en_val: &'static str,
    ja_key: &'static str,
    ja_val: &'static str,
    ko_key: &'static str,
    ko_val: &'static str,
) {
    let mut langs = HashMap::new();
    langs.insert(zh_key, zh_val);
    langs.insert(en_key, en_val);
    langs.insert(ja_key, ja_val);
    langs.insert(ko_key, ko_val);
    dict.insert(key, langs);
}

/// 获取翻译文本
pub fn get_text(key: &str, lang: &str) -> String {
    TRANSLATIONS
        .get(key)
        .and_then(|langs: &HashMap<&str, &str>| langs.get(lang))
        .map(|s: &&str| s.to_string())
        .unwrap_or_else(|| {
            // 降级：尝试获取中文
            TRANSLATIONS
                .get(key)
                .and_then(|langs: &HashMap<&str, &str>| langs.get("zh"))
                .map(|s: &&str| s.to_string())
                .unwrap_or_else(|| {
                    // 最终降级：返回 key 本身
                    #[cfg(debug_assertions)]
                    web_sys::console::warn_1(
                        &format!("Missing translation for key: {} (lang: {})", key, lang).into(),
                    );
                    key.to_string()
                })
        })
}
