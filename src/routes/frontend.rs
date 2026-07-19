use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, Key, PrivateCookieJar, SameSite};

use crate::{
    app::AppState,
    auth::user::{AUTH_COOKIE_NAME, User},
    error::{AppError, FieldError},
    models::{Asset, AuthForm, CreateAssetRequest, PortfolioSummary, UpdateAssetRequest},
    repository::Repository,
    services::{auth::AuthService, portfolio::PortfolioService},
};

pub fn router() -> Router<AppState> {
    Router::new()
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
    summary: PortfolioSummary,
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

async fn index(user: User, repository: Repository) -> Result<Html<String>, AppError> {
    let summary = PortfolioService::new(repository).summary(&user).await?;
    let html = DashboardPage {
        user_email: user.email().clone(),
        summary,
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
