use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, Key, PrivateCookieJar, SameSite};
use rust_decimal::Decimal;

use crate::{
    app::AppState,
    auth::user::{AUTH_COOKIE_NAME, User},
    error::{AppError, FieldError},
    models::{Asset, AssetMovement, AuthForm, CreateAssetRequest, UpdateAssetRequest},
    repository::Repository,
    services::{auth::AuthService, portfolio::PortfolioService},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/assets/azurewlogo.svg", get(azure_wallet_logo))
        .route("/assets/azurewlogo-white.svg", get(azure_wallet_logo_white))
        .route("/assets/icon.svg", get(azure_wallet_icon))
        .route("/favicon.svg", get(favicon_svg))
        .route("/favicon-96x96.png", get(favicon_png))
        .route("/favicon.ico", get(favicon_ico))
        .route("/apple-touch-icon.png", get(apple_touch_icon))
        .route(
            "/web-app-manifest-192x192.png",
            get(web_app_manifest_icon_192),
        )
        .route(
            "/web-app-manifest-512x512.png",
            get(web_app_manifest_icon_512),
        )
        .route("/site.webmanifest", get(site_webmanifest))
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
        .route("/register", get(register_page).post(register))
        .route("/logout", post(logout))
        .route("/assets/new", get(new_asset_page).post(create_asset))
        .route(
            "/assets/{asset_id}/edit",
            get(edit_asset_page).post(update_asset),
        )
}

async fn azure_wallet_logo() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        include_str!("../assets/AzureWLogo.svg"),
    )
}

async fn azure_wallet_logo_white() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        include_str!("../assets/AzureWLogowhite.svg"),
    )
}

async fn azure_wallet_icon() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        include_str!("../assets/icon.svg"),
    )
}

async fn favicon_svg() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        include_str!("../assets/favicons/favicon.svg"),
    )
}

async fn favicon_png() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/png")],
        include_bytes!("../assets/favicons/favicon-96x96.png").as_slice(),
    )
}

async fn favicon_ico() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/x-icon")],
        include_bytes!("../assets/favicons/favicon.ico").as_slice(),
    )
}

async fn apple_touch_icon() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/png")],
        include_bytes!("../assets/favicons/apple-touch-icon.png").as_slice(),
    )
}

async fn web_app_manifest_icon_192() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/png")],
        include_bytes!("../assets/favicons/web-app-manifest-192x192.png").as_slice(),
    )
}

async fn web_app_manifest_icon_512() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/png")],
        include_bytes!("../assets/favicons/web-app-manifest-512x512.png").as_slice(),
    )
}

async fn site_webmanifest() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/manifest+json; charset=utf-8")],
        include_str!("../assets/favicons/site.webmanifest"),
    )
}

#[derive(Template)]
#[template(path = "auth.html")]
struct AuthPage {
    mode: &'static str,
    action: &'static str,
    title: &'static str,
    subtitle: &'static str,
    button: &'static str,
    alternate_href: &'static str,
    alternate_text: &'static str,
    alternate_link: &'static str,
    email: String,
    general_error: Option<String>,
    email_error: Option<String>,
    password_error: Option<String>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardPage {
    user_email: String,
    total_value_display: String,
    selected_currency: String,
    currency_options: Vec<CurrencyOption>,
    assets: Vec<AssetRow>,
    movements: Vec<MovementRow>,
}

#[derive(Template)]
#[template(path = "asset_form.html")]
struct AssetFormPage {
    title: &'static str,
    action: String,
    button: &'static str,
    form: AssetFormData,
    general_error: Option<String>,
    name_error: Option<String>,
    ticker_error: Option<String>,
    asset_class_error: Option<String>,
    quantity_error: Option<String>,
    average_price_error: Option<String>,
    current_price_error: Option<String>,
    currency_error: Option<String>,
}

#[derive(Default)]
struct AssetFormData {
    name: String,
    ticker: String,
    asset_class: String,
    quantity: String,
    average_price: String,
    current_price: String,
    currency: String,
}

#[derive(serde::Deserialize)]
struct DashboardQuery {
    display_currency: Option<String>,
}

struct CurrencyOption {
    code: String,
    country: String,
    flag: String,
    selected: bool,
}

struct AssetRow {
    id: i64,
    name: String,
    ticker: String,
    asset_class: String,
    currency: String,
    quantity: String,
    current_price_display: String,
    total_value_display: String,
}

struct MovementRow {
    asset_name: String,
    ticker: String,
    occurred_at: String,
    is_entry: bool,
    movement_label: &'static str,
    quantity_display: String,
    total_value_display: String,
}

async fn login_page() -> Result<Html<String>, AppError> {
    render_auth(AuthPage::login(String::new(), Vec::new(), None))
}

async fn register_page() -> Result<Html<String>, AppError> {
    render_auth(AuthPage::register(String::new(), Vec::new(), None))
}

async fn login(
    State(state): State<AppState>,
    repository: Repository,
    jar: PrivateCookieJar<Key>,
    Form(form): Form<AuthForm>,
) -> Result<Response, AppError> {
    match AuthService::new(repository).login(form.clone()).await {
        Ok(user) => {
            let token = user.auth_token(&state)?;
            Ok((jar.add(auth_cookie(token, &state)), Redirect::to("/")).into_response())
        }
        Err(err) => render_auth_response(AuthPage::login(
            form.email,
            err.field_errors(),
            Some(err.to_string()),
        )),
    }
}

async fn register(
    State(state): State<AppState>,
    repository: Repository,
    jar: PrivateCookieJar<Key>,
    Form(form): Form<AuthForm>,
) -> Result<Response, AppError> {
    match AuthService::new(repository).register(form.clone()).await {
        Ok(user) => {
            let token = user.auth_token(&state)?;
            Ok((jar.add(auth_cookie(token, &state)), Redirect::to("/")).into_response())
        }
        Err(err) => render_auth_response(AuthPage::register(
            form.email,
            err.field_errors(),
            Some(err.to_string()),
        )),
    }
}

async fn logout(jar: PrivateCookieJar<Key>) -> impl IntoResponse {
    (
        jar.remove(Cookie::build(AUTH_COOKIE_NAME).path("/").build()),
        Redirect::to("/login"),
    )
}

async fn index(
    Query(query): Query<DashboardQuery>,
    user: User,
    repository: Repository,
) -> Result<Html<String>, AppError> {
    let display_currency = normalize_display_currency(query.display_currency);
    let service = PortfolioService::new(repository);
    let summary = service.summary(&user).await?;
    let movements = service.movements(&user).await?;
    let total_value = summary
        .assets
        .iter()
        .map(|asset| convert_currency(asset.total_value, &asset.currency, &display_currency))
        .sum();

    let html = DashboardPage {
        user_email: user.email().clone(),
        total_value_display: format_money(total_value, &display_currency),
        selected_currency: display_currency.clone(),
        currency_options: currency_options(&display_currency),
        assets: summary
            .assets
            .iter()
            .map(|asset| AssetRow::from_asset(asset, &display_currency))
            .collect(),
        movements: movements
            .iter()
            .map(|movement| MovementRow::from_movement(movement, &display_currency))
            .collect(),
    }
    .render()?;

    Ok(Html(html))
}

async fn new_asset_page() -> Result<Html<String>, AppError> {
    render_asset_form(AssetFormPage::new_asset(
        AssetFormData::default(),
        Vec::new(),
        None,
    ))
}

async fn create_asset(
    user: User,
    repository: Repository,
    Form(request): Form<CreateAssetRequest>,
) -> Result<Response, AppError> {
    let form = AssetFormData::from_request(&request);

    match PortfolioService::new(repository)
        .create_asset(&user, request)
        .await
    {
        Ok(_) => Ok(Redirect::to("/").into_response()),
        Err(err) => render_asset_form_response(AssetFormPage::new_asset(
            form,
            err.field_errors(),
            Some(err.to_string()),
        )),
    }
}

async fn edit_asset_page(
    user: User,
    repository: Repository,
    Path(asset_id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let asset = PortfolioService::new(repository)
        .get_asset(&user, asset_id)
        .await?;

    render_asset_form(AssetFormPage::edit_asset(&asset, Vec::new(), None))
}

async fn update_asset(
    user: User,
    repository: Repository,
    Path(asset_id): Path<i64>,
    Form(request): Form<CreateAssetRequest>,
) -> Result<Response, AppError> {
    let form = AssetFormData::from_request(&request);
    let update = UpdateAssetRequest {
        name: Some(request.name),
        ticker: Some(request.ticker),
        asset_class: Some(request.asset_class),
        quantity: Some(request.quantity),
        average_price: Some(request.average_price),
        current_price: Some(request.current_price),
        currency: Some(request.currency),
    };

    match PortfolioService::new(repository)
        .update_asset(&user, asset_id, update)
        .await
    {
        Ok(_) => Ok(Redirect::to("/").into_response()),
        Err(err) => render_asset_form_response(AssetFormPage::edit_asset_from_form(
            asset_id,
            form,
            err.field_errors(),
            Some(err.to_string()),
        )),
    }
}

impl AuthPage {
    fn login(email: String, errors: Vec<FieldError>, general_error: Option<String>) -> Self {
        Self {
            mode: "login",
            action: "/login",
            title: "Welcome back",
            subtitle: "Sign in to see your movements and investments.",
            button: "Sign in",
            alternate_href: "/register",
            alternate_text: "New here?",
            alternate_link: "Create an account",
            email,
            general_error,
            email_error: error_for(&errors, "email"),
            password_error: error_for(&errors, "password"),
        }
    }

    fn register(email: String, errors: Vec<FieldError>, general_error: Option<String>) -> Self {
        Self {
            mode: "register",
            action: "/register",
            title: "Create account",
            subtitle: "Register to start tracking your investments.",
            button: "Register",
            alternate_href: "/login",
            alternate_text: "Already registered?",
            alternate_link: "Sign in",
            email,
            general_error,
            email_error: error_for(&errors, "email"),
            password_error: error_for(&errors, "password"),
        }
    }
}

impl AssetFormPage {
    fn new_asset(
        form: AssetFormData,
        errors: Vec<FieldError>,
        general_error: Option<String>,
    ) -> Self {
        Self::from_parts(
            "New asset",
            "/assets/new".to_string(),
            "Create asset",
            form,
            errors,
            general_error,
        )
    }

    fn edit_asset(asset: &Asset, errors: Vec<FieldError>, general_error: Option<String>) -> Self {
        Self::from_parts(
            "Edit asset",
            format!("/assets/{}/edit", asset.id),
            "Save asset",
            AssetFormData::from_asset(asset),
            errors,
            general_error,
        )
    }

    fn edit_asset_from_form(
        asset_id: i64,
        form: AssetFormData,
        errors: Vec<FieldError>,
        general_error: Option<String>,
    ) -> Self {
        Self::from_parts(
            "Edit asset",
            format!("/assets/{asset_id}/edit"),
            "Save asset",
            form,
            errors,
            general_error,
        )
    }

    fn from_parts(
        title: &'static str,
        action: String,
        button: &'static str,
        form: AssetFormData,
        errors: Vec<FieldError>,
        general_error: Option<String>,
    ) -> Self {
        Self {
            title,
            action,
            button,
            form,
            general_error,
            name_error: error_for(&errors, "name"),
            ticker_error: error_for(&errors, "ticker"),
            asset_class_error: error_for(&errors, "asset_class"),
            quantity_error: error_for(&errors, "quantity"),
            average_price_error: error_for(&errors, "average_price"),
            current_price_error: error_for(&errors, "current_price"),
            currency_error: error_for(&errors, "currency"),
        }
    }
}

impl AssetFormData {
    fn from_request(request: &CreateAssetRequest) -> Self {
        Self {
            name: request.name.clone(),
            ticker: request.ticker.clone(),
            asset_class: request.asset_class.clone(),
            quantity: request.quantity.to_string(),
            average_price: request.average_price.to_string(),
            current_price: request.current_price.to_string(),
            currency: request.currency.clone(),
        }
    }

    fn from_asset(asset: &Asset) -> Self {
        Self {
            name: asset.name.clone(),
            ticker: asset.ticker.clone(),
            asset_class: asset.asset_class.clone(),
            quantity: asset.quantity.to_string(),
            average_price: asset.average_price.to_string(),
            current_price: asset.current_price.to_string(),
            currency: asset.currency.clone(),
        }
    }
}

impl AssetRow {
    fn from_asset(asset: &Asset, display_currency: &str) -> Self {
        Self {
            id: asset.id,
            name: asset.name.clone(),
            ticker: asset.ticker.clone(),
            asset_class: asset.asset_class.clone(),
            currency: asset.currency.clone(),
            quantity: asset.quantity.to_string(),
            current_price_display: format_money(asset.current_price, &asset.currency),
            total_value_display: format_money(
                convert_currency(asset.total_value, &asset.currency, display_currency),
                display_currency,
            ),
        }
    }
}

impl MovementRow {
    fn from_movement(movement: &AssetMovement, display_currency: &str) -> Self {
        let is_entry = movement.movement_type == "entry";
        let sign = if is_entry { "+" } else { "-" };

        Self {
            asset_name: movement.asset_name.clone(),
            ticker: movement.ticker.clone(),
            occurred_at: movement.occurred_at.clone(),
            is_entry,
            movement_label: if is_entry { "Entry" } else { "Exit" },
            quantity_display: format!("{sign}{}", movement.quantity),
            total_value_display: format_money(
                convert_currency(movement.total_value, &movement.currency, display_currency),
                display_currency,
            ),
        }
    }
}

fn normalize_display_currency(display_currency: Option<String>) -> String {
    let currency = display_currency
        .unwrap_or_else(|| "BRL".to_string())
        .trim()
        .to_uppercase();

    if currency_rate_to_usd(&currency).is_some() {
        currency
    } else {
        "BRL".to_string()
    }
}

fn currency_options(selected_currency: &str) -> Vec<CurrencyOption> {
    [
        ("BRL", "Brazil", "\u{1F1E7}\u{1F1F7}"),
        ("USD", "United States", "\u{1F1FA}\u{1F1F8}"),
        ("EUR", "European Union", "\u{1F1EA}\u{1F1FA}"),
        ("GBP", "United Kingdom", "\u{1F1EC}\u{1F1E7}"),
        ("BTC", "Bitcoin", "₿"),
        ("ETH", "Ethereum", "◇"),
    ]
    .into_iter()
    .map(|(code, country, flag)| CurrencyOption {
        code: code.to_string(),
        country: country.to_string(),
        flag: flag.to_string(),
        selected: code == selected_currency,
    })
    .collect()
}

fn convert_currency(amount: Decimal, source_currency: &str, target_currency: &str) -> Decimal {
    let source_rate = currency_rate_to_usd(source_currency).unwrap_or(Decimal::new(1, 0));
    let target_rate = currency_rate_to_usd(target_currency).unwrap_or(Decimal::new(1, 0));

    (amount * source_rate / target_rate).round_dp(2)
}

fn currency_rate_to_usd(currency: &str) -> Option<Decimal> {
    match currency {
        "BRL" => Some(Decimal::new(18, 2)),
        "USD" => Some(Decimal::new(1, 0)),
        "EUR" => Some(Decimal::new(109, 2)),
        "GBP" => Some(Decimal::new(127, 2)),
        "BTC" => Some(Decimal::new(65000, 0)),
        "ETH" => Some(Decimal::new(3500, 0)),
        _ => None,
    }
}

fn format_money(amount: Decimal, currency: &str) -> String {
    let rounded = amount.round_dp(2);
    let amount_text = add_grouping(&format!("{rounded:.2}"));

    match currency {
        "BRL" => format!("R$ {amount_text}"),
        "USD" => format!("US$ {amount_text}"),
        "EUR" => format!("€ {amount_text}"),
        "GBP" => format!("£ {amount_text}"),
        "BTC" => format!("₿ {amount_text}"),
        "ETH" => format!("Ξ {amount_text}"),
        _ => format!("{currency} {amount_text}"),
    }
}

fn add_grouping(value: &str) -> String {
    let (whole, cents) = value.split_once('.').unwrap_or((value, "00"));
    let mut grouped = String::new();

    for (index, character) in whole.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(character);
    }

    let whole = grouped.chars().rev().collect::<String>();
    format!("{whole}.{cents}")
}

fn render_auth(page: AuthPage) -> Result<Html<String>, AppError> {
    Ok(Html(page.render()?))
}

fn render_auth_response(page: AuthPage) -> Result<Response, AppError> {
    Ok((StatusCode::BAD_REQUEST, Html(page.render()?)).into_response())
}

fn render_asset_form(page: AssetFormPage) -> Result<Html<String>, AppError> {
    Ok(Html(page.render()?))
}

fn render_asset_form_response(page: AssetFormPage) -> Result<Response, AppError> {
    Ok((StatusCode::BAD_REQUEST, Html(page.render()?)).into_response())
}

fn error_for(errors: &[FieldError], field: &str) -> Option<String> {
    errors
        .iter()
        .find(|error| error.field == field)
        .map(|error| error.message.clone())
}

fn auth_cookie(token: String, state: &AppState) -> Cookie<'static> {
    Cookie::build((AUTH_COOKIE_NAME, token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(state.auth.cookie_secure)
        .path("/")
        .build()
}
