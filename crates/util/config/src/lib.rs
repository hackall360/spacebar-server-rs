use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::OnceCell;

static CONFIG: OnceCell<Arc<Config>> = OnceCell::const_new();

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub gateway: EndpointConfiguration,
    pub cdn: CdnConfiguration,
    pub api: ApiConfiguration,
    pub general: GeneralConfiguration,
    pub limits: LimitsConfiguration,
    pub security: SecurityConfiguration,
    pub login: LoginConfiguration,
    pub register: RegisterConfiguration,
    pub regions: RegionConfiguration,
    pub guild: GuildConfiguration,
    pub gif: GifConfiguration,
    pub rabbitmq: RabbitMQConfiguration,
    pub kafka: KafkaConfiguration,
    pub templates: TemplateConfiguration,
    pub metrics: MetricsConfiguration,
    pub sentry: SentryConfiguration,
    pub defaults: DefaultsConfiguration,
    pub external: ExternalTokensConfiguration,
    pub email: EmailConfiguration,
    #[serde(rename = "passwordReset")]
    pub password_reset: PasswordResetConfiguration,
    pub user: UserConfiguration,
}

impl Config {
    pub async fn init() -> Arc<Self> {
        CONFIG
            .get_or_init(|| async {
                let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.json".to_string());
                let cfg = match tokio::fs::read_to_string(&path).await {
                    Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                    Err(_) => Self::default(),
                };
                Arc::new(cfg)
            })
            .await
            .clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct EndpointConfiguration {
    pub endpoint_client: Option<String>,
    pub endpoint_private: Option<String>,
    pub endpoint_public: Option<String>,
}
impl Default for EndpointConfiguration {
    fn default() -> Self {
        Self {
            endpoint_client: None,
            endpoint_private: None,
            endpoint_public: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct CdnConfiguration {
    #[serde(flatten)]
    pub endpoint: EndpointConfiguration,
    pub resize_height_max: u32,
    pub resize_width_max: u32,
    pub imagor_server_url: Option<String>,
    pub proxy_cache_header_seconds: u32,
}
impl Default for CdnConfiguration {
    fn default() -> Self {
        Self {
            endpoint: EndpointConfiguration::default(),
            resize_height_max: 1000,
            resize_width_max: 1000,
            imagor_server_url: None,
            proxy_cache_header_seconds: 60 * 60 * 24,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ApiConfiguration {
    pub default_version: String,
    pub active_versions: Vec<String>,
    pub endpoint_public: Option<String>,
}
impl Default for ApiConfiguration {
    fn default() -> Self {
        Self {
            default_version: "9".into(),
            active_versions: vec!["6".into(), "7".into(), "8".into(), "9".into()],
            endpoint_public: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct GeneralConfiguration {
    pub instance_name: String,
    pub instance_description: Option<String>,
    pub front_page: Option<String>,
    pub tos_page: Option<String>,
    pub correspondence_email: Option<String>,
    pub correspondence_user_id: Option<String>,
    pub image: Option<String>,
    pub instance_id: String,
    pub auto_create_bot_users: bool,
}
impl Default for GeneralConfiguration {
    fn default() -> Self {
        Self {
            instance_name: "Spacebar Instance".into(),
            instance_description: Some(
                "This is a Spacebar instance made in the pre-release days".into(),
            ),
            front_page: None,
            tos_page: None,
            correspondence_email: None,
            correspondence_user_id: None,
            image: None,
            instance_id: "0".into(),
            auto_create_bot_users: false,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct GifConfiguration {
    pub enabled: bool,
    pub provider: String,
    pub api_key: Option<String>,
}
impl Default for GifConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: "tenor".into(),
            api_key: Some("LIVDSRZULELA".into()),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct RabbitMQConfiguration {
    pub host: Option<String>,
}
impl Default for RabbitMQConfiguration {
    fn default() -> Self {
        Self { host: None }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct KafkaConfiguration {
    pub brokers: Option<Vec<KafkaBroker>>,
}
impl Default for KafkaConfiguration {
    fn default() -> Self {
        Self { brokers: None }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaBroker {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct TemplateConfiguration {
    pub enabled: bool,
    pub allow_template_creation: bool,
    pub allow_discord_templates: bool,
    pub allow_raws: bool,
}
impl Default for TemplateConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            allow_template_creation: true,
            allow_discord_templates: true,
            allow_raws: true,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct MetricsConfiguration {
    pub timeout: u32,
}
impl Default for MetricsConfiguration {
    fn default() -> Self {
        Self { timeout: 30000 }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct SentryConfiguration {
    pub enabled: bool,
    pub endpoint: String,
    pub trace_sample_rate: f32,
    pub environment: Option<String>,
}
impl Default for SentryConfiguration {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: "https://05e8e3d005f34b7d97e920ae5870a5e5@sentry.thearcanebrony.net/6"
                .into(),
            trace_sample_rate: 1.0,
            environment: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct DefaultsConfiguration {
    pub guild: GuildDefaults,
    pub user: UserDefaults,
}
impl Default for DefaultsConfiguration {
    fn default() -> Self {
        Self {
            guild: GuildDefaults::default(),
            user: UserDefaults::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct GuildDefaults {
    pub max_presences: u32,
    pub max_video_channel_users: u32,
    pub afk_timeout: u32,
    pub default_message_notifications: u32,
    pub explicit_content_filter: u32,
}
impl Default for GuildDefaults {
    fn default() -> Self {
        Self {
            max_presences: 250000,
            max_video_channel_users: 200,
            afk_timeout: 300,
            default_message_notifications: 1,
            explicit_content_filter: 0,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct UserDefaults {
    pub premium: bool,
    pub premium_type: u32,
    pub verified: bool,
}
impl Default for UserDefaults {
    fn default() -> Self {
        Self {
            premium: true,
            premium_type: 2,
            verified: true,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ExternalTokensConfiguration {
    pub twitter: Option<String>,
}
impl Default for ExternalTokensConfiguration {
    fn default() -> Self {
        Self { twitter: None }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct EmailConfiguration {
    pub provider: Option<String>,
    pub sender_address: Option<String>,
    pub smtp: SMTPConfiguration,
    pub mailgun: MailGunConfiguration,
    pub mailjet: MailJetConfiguration,
    pub sendgrid: SendGridConfiguration,
}
impl Default for EmailConfiguration {
    fn default() -> Self {
        Self {
            provider: None,
            sender_address: None,
            smtp: SMTPConfiguration::default(),
            mailgun: MailGunConfiguration::default(),
            mailjet: MailJetConfiguration::default(),
            sendgrid: SendGridConfiguration::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct SMTPConfiguration {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub secure: Option<bool>,
    pub username: Option<String>,
    pub password: Option<String>,
}
impl Default for SMTPConfiguration {
    fn default() -> Self {
        Self {
            host: None,
            port: None,
            secure: None,
            username: None,
            password: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct MailGunConfiguration {
    pub api_key: Option<String>,
    pub domain: Option<String>,
}
impl Default for MailGunConfiguration {
    fn default() -> Self {
        Self {
            api_key: None,
            domain: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct MailJetConfiguration {
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
}
impl Default for MailJetConfiguration {
    fn default() -> Self {
        Self {
            api_key: None,
            api_secret: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct SendGridConfiguration {
    pub api_key: Option<String>,
}
impl Default for SendGridConfiguration {
    fn default() -> Self {
        Self { api_key: None }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct PasswordResetConfiguration {
    pub require_captcha: bool,
}
impl Default for PasswordResetConfiguration {
    fn default() -> Self {
        Self {
            require_captcha: false,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct UserConfiguration {
    pub blocked_contains: Vec<String>,
    pub blocked_equals: Vec<String>,
}
impl Default for UserConfiguration {
    fn default() -> Self {
        Self {
            blocked_contains: vec!["discord".into(), "clyde".into(), "spacebar".into()],
            blocked_equals: vec!["everyone".into(), "here".into()],
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct RegionConfiguration {
    pub default: String,
    pub use_default_as_optimal: bool,
    pub available: Vec<Region>,
}
impl Default for RegionConfiguration {
    fn default() -> Self {
        Self {
            default: "spacebar".into(),
            use_default_as_optimal: true,
            available: vec![Region::default()],
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Region {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub location: Option<Location>,
    pub vip: bool,
    pub custom: bool,
    pub deprecated: bool,
}
impl Default for Region {
    fn default() -> Self {
        Self {
            id: "spacebar".into(),
            name: "spacebar".into(),
            endpoint: "127.0.0.1:3004".into(),
            location: None,
            vip: false,
            custom: false,
            deprecated: false,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}
impl Default for Location {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct GuildConfiguration {
    pub discovery: DiscoveryConfiguration,
    #[serde(rename = "autoJoin")]
    pub auto_join: AutoJoinConfiguration,
    #[serde(rename = "defaultFeatures")]
    pub default_features: Vec<String>,
}
impl Default for GuildConfiguration {
    fn default() -> Self {
        Self {
            discovery: DiscoveryConfiguration::default(),
            auto_join: AutoJoinConfiguration::default(),
            default_features: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct DiscoveryConfiguration {
    pub show_all_guilds: bool,
    pub use_recommendation: bool,
    pub offset: u32,
    pub limit: u32,
}
impl Default for DiscoveryConfiguration {
    fn default() -> Self {
        Self {
            show_all_guilds: false,
            use_recommendation: false,
            offset: 0,
            limit: 24,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct AutoJoinConfiguration {
    pub enabled: bool,
    pub guilds: Vec<String>,
    #[serde(rename = "canLeave")]
    pub can_leave: bool,
}
impl Default for AutoJoinConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            guilds: Vec::new(),
            can_leave: true,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct LoginConfiguration {
    pub require_captcha: bool,
    pub require_verification: bool,
}
impl Default for LoginConfiguration {
    fn default() -> Self {
        Self {
            require_captcha: false,
            require_verification: false,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct RegisterConfiguration {
    pub email: RegistrationEmailConfiguration,
    #[serde(rename = "dateOfBirth")]
    pub date_of_birth: DateOfBirthConfiguration,
    pub password: PasswordConfiguration,
    pub disabled: bool,
    #[serde(rename = "requireCaptcha")]
    pub require_captcha: bool,
    #[serde(rename = "requireInvite")]
    pub require_invite: bool,
    #[serde(rename = "guestsRequireInvite")]
    pub guests_require_invite: bool,
    #[serde(rename = "allowNewRegistration")]
    pub allow_new_registration: bool,
    #[serde(rename = "allowMultipleAccounts")]
    pub allow_multiple_accounts: bool,
    #[serde(rename = "blockProxies")]
    pub block_proxies: bool,
    #[serde(rename = "incrementingDiscriminators")]
    pub incrementing_discriminators: bool,
    #[serde(rename = "defaultRights")]
    pub default_rights: String,
}
impl Default for RegisterConfiguration {
    fn default() -> Self {
        Self {
            email: RegistrationEmailConfiguration::default(),
            date_of_birth: DateOfBirthConfiguration::default(),
            password: PasswordConfiguration::default(),
            disabled: false,
            require_captcha: true,
            require_invite: false,
            guests_require_invite: true,
            allow_new_registration: true,
            allow_multiple_accounts: true,
            block_proxies: true,
            incrementing_discriminators: false,
            default_rights: "875069521787904".into(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct RegistrationEmailConfiguration {
    pub required: bool,
    pub allowlist: bool,
    pub blocklist: bool,
    pub domains: Vec<String>,
}
impl Default for RegistrationEmailConfiguration {
    fn default() -> Self {
        Self {
            required: false,
            allowlist: false,
            blocklist: true,
            domains: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct DateOfBirthConfiguration {
    pub required: bool,
    pub minimum: u32,
}
impl Default for DateOfBirthConfiguration {
    fn default() -> Self {
        Self {
            required: true,
            minimum: 13,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct PasswordConfiguration {
    pub required: bool,
    #[serde(rename = "minLength")]
    pub min_length: u32,
    #[serde(rename = "minNumbers")]
    pub min_numbers: u32,
    #[serde(rename = "minUpperCase")]
    pub min_upper_case: u32,
    #[serde(rename = "minSymbols")]
    pub min_symbols: u32,
}
impl Default for PasswordConfiguration {
    fn default() -> Self {
        Self {
            required: false,
            min_length: 8,
            min_numbers: 2,
            min_upper_case: 2,
            min_symbols: 0,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct SecurityConfiguration {
    pub captcha: CaptchaConfiguration,
    #[serde(rename = "twoFactor")]
    pub two_factor: TwoFactorConfiguration,
    #[serde(rename = "autoUpdate")]
    pub auto_update: AutoUpdate,
    #[serde(rename = "requestSignature")]
    pub request_signature: String,
    #[serde(rename = "jwtSecret")]
    pub jwt_secret: String,
    #[serde(rename = "forwardedFor")]
    pub forwarded_for: Option<String>,
    #[serde(rename = "trustedProxies")]
    pub trusted_proxies: Option<serde_json::Value>,
    #[serde(rename = "ipdataApiKey")]
    pub ipdata_api_key: Option<String>,
    #[serde(rename = "mfaBackupCodeCount")]
    pub mfa_backup_code_count: u32,
    #[serde(rename = "statsWorldReadable")]
    pub stats_world_readable: bool,
    #[serde(rename = "defaultRegistrationTokenExpiration")]
    pub default_registration_token_expiration: u64,
    #[serde(rename = "cdnSignUrls")]
    pub cdn_sign_urls: bool,
    #[serde(rename = "cdnSignatureKey")]
    pub cdn_signature_key: String,
    #[serde(rename = "cdnSignatureDuration")]
    pub cdn_signature_duration: String,
    #[serde(rename = "cdnSignatureIncludeIp")]
    pub cdn_signature_include_ip: bool,
    #[serde(rename = "cdnSignatureIncludeUserAgent")]
    pub cdn_signature_include_user_agent: bool,
}
impl Default for SecurityConfiguration {
    fn default() -> Self {
        Self {
            captcha: CaptchaConfiguration::default(),
            two_factor: TwoFactorConfiguration::default(),
            auto_update: AutoUpdate::Bool(true),
            request_signature: String::new(),
            jwt_secret: String::new(),
            forwarded_for: None,
            trusted_proxies: None,
            ipdata_api_key: Some(
                "eca677b284b3bac29eb72f5e496aa9047f26543605efe99ff2ce35c9".into(),
            ),
            mfa_backup_code_count: 10,
            stats_world_readable: true,
            default_registration_token_expiration: 1000 * 60 * 60 * 24 * 7,
            cdn_sign_urls: false,
            cdn_signature_key: String::new(),
            cdn_signature_duration: "24h".into(),
            cdn_signature_include_ip: true,
            cdn_signature_include_user_agent: true,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct CaptchaConfiguration {
    pub enabled: bool,
    pub service: Option<String>,
    pub sitekey: Option<String>,
    pub secret: Option<String>,
}
impl Default for CaptchaConfiguration {
    fn default() -> Self {
        Self {
            enabled: false,
            service: None,
            sitekey: None,
            secret: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct TwoFactorConfiguration {
    #[serde(rename = "generateBackupCodes")]
    pub generate_backup_codes: bool,
}
impl Default for TwoFactorConfiguration {
    fn default() -> Self {
        Self {
            generate_backup_codes: true,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum AutoUpdate {
    Bool(bool),
    Number(u64),
}
impl Default for AutoUpdate {
    fn default() -> Self {
        AutoUpdate::Bool(true)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct LimitsConfiguration {
    pub user: UserLimits,
    pub guild: GuildLimits,
    pub message: MessageLimits,
    pub channel: ChannelLimits,
    pub rate: RateLimits,
    #[serde(rename = "absoluteRate")]
    pub absolute_rate: GlobalRateLimits,
}
impl Default for LimitsConfiguration {
    fn default() -> Self {
        Self {
            user: UserLimits::default(),
            guild: GuildLimits::default(),
            message: MessageLimits::default(),
            channel: ChannelLimits::default(),
            rate: RateLimits::default(),
            absolute_rate: GlobalRateLimits::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct UserLimits {
    pub max_guilds: u32,
    pub max_username: u32,
    pub max_friends: u32,
    pub max_bio: u32,
}
impl Default for UserLimits {
    fn default() -> Self {
        Self {
            max_guilds: 1048576,
            max_username: 32,
            max_friends: 5000,
            max_bio: 190,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct GuildLimits {
    pub max_roles: u32,
    pub max_emojis: u32,
    pub max_members: u64,
    pub max_channels: u32,
    pub max_bulk_ban_users: u32,
    pub max_channels_in_category: u32,
}
impl Default for GuildLimits {
    fn default() -> Self {
        Self {
            max_roles: 1000,
            max_emojis: 2000,
            max_members: 25_000_000,
            max_channels: 65535,
            max_bulk_ban_users: 200,
            max_channels_in_category: 65535,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct MessageLimits {
    pub max_characters: u32,
    pub max_tts_characters: u32,
    pub max_reactions: u32,
    pub max_attachment_size: u64,
    pub max_bulk_delete: u32,
    pub max_embed_download_size: u64,
}
impl Default for MessageLimits {
    fn default() -> Self {
        Self {
            max_characters: 1_048_576,
            max_tts_characters: 160,
            max_reactions: 2048,
            max_attachment_size: 1024 * 1024 * 1024,
            max_bulk_delete: 1000,
            max_embed_download_size: 1024 * 1024 * 5,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ChannelLimits {
    pub max_pins: u32,
    pub max_topic: u32,
    pub max_webhooks: u32,
}
impl Default for ChannelLimits {
    fn default() -> Self {
        Self {
            max_pins: 500,
            max_topic: 1024,
            max_webhooks: 100,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct RateLimits {
    pub enabled: bool,
    pub ip: RateLimitOptions,
    pub global: RateLimitOptions,
    pub error: RateLimitOptions,
    pub routes: RouteRateLimit,
}
impl Default for RateLimits {
    fn default() -> Self {
        Self {
            enabled: false,
            ip: RateLimitOptions {
                count: 500,
                window: 5,
                ..Default::default()
            },
            global: RateLimitOptions {
                count: 250,
                window: 5,
                ..Default::default()
            },
            error: RateLimitOptions {
                count: 10,
                window: 5,
                ..Default::default()
            },
            routes: RouteRateLimit::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct RateLimitOptions {
    pub bot: Option<u32>,
    pub count: u32,
    pub window: u32,
    #[serde(rename = "onyIp")]
    pub ony_ip: Option<bool>,
}
impl Default for RateLimitOptions {
    fn default() -> Self {
        Self {
            bot: None,
            count: 0,
            window: 0,
            ony_ip: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct RouteRateLimit {
    pub guild: RateLimitOptions,
    pub webhook: RateLimitOptions,
    pub channel: RateLimitOptions,
    pub auth: AuthRateLimit,
}
impl Default for RouteRateLimit {
    fn default() -> Self {
        Self {
            guild: RateLimitOptions {
                count: 5,
                window: 5,
                ..Default::default()
            },
            webhook: RateLimitOptions {
                count: 10,
                window: 5,
                ..Default::default()
            },
            channel: RateLimitOptions {
                count: 10,
                window: 5,
                ..Default::default()
            },
            auth: AuthRateLimit::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct AuthRateLimit {
    pub login: RateLimitOptions,
    pub register: RateLimitOptions,
}
impl Default for AuthRateLimit {
    fn default() -> Self {
        Self {
            login: RateLimitOptions {
                count: 5,
                window: 60,
                ..Default::default()
            },
            register: RateLimitOptions {
                count: 2,
                window: 60 * 60 * 12,
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct GlobalRateLimits {
    pub register: GlobalRateLimit,
    #[serde(rename = "sendMessage")]
    pub send_message: GlobalRateLimit,
}
impl Default for GlobalRateLimits {
    fn default() -> Self {
        Self {
            register: GlobalRateLimit {
                limit: 25,
                window: 60 * 60 * 1000,
                enabled: true,
            },
            send_message: GlobalRateLimit {
                limit: 200,
                window: 60 * 1000,
                enabled: true,
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct GlobalRateLimit {
    pub limit: u32,
    pub window: u32,
    pub enabled: bool,
}
impl Default for GlobalRateLimit {
    fn default() -> Self {
        Self {
            limit: 100,
            window: 60 * 60 * 1000,
            enabled: true,
        }
    }
}
