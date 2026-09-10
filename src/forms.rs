//! Angular-Reactive-Forms-inspired form state: [`FormBuilder`] declares a
//! [`Form`]'s [`FormControl`]s (name, initial value, validators), the
//! component owns the built `Form` in its own state, and the `<Form>`/
//! `formControl` template attributes (see `parser.rs`/`eval.rs`) bind inputs
//! to it by name — mirroring Angular's `FormGroup`/`formControlName`.
//!
//! ```
//! use glacier_ui::{FormBuilder, FormControl};
//!
//! let mut form = FormBuilder::new("login")
//!     .control(FormControl::new("username", "").required().min_length(3))
//!     .control(FormControl::new("password", "").required().min_length(6))
//!     .build();
//!
//! assert!(!form.is_valid());
//! form.set_value("username", "ana");
//! form.set_value("password", "hunter2");
//! assert!(form.is_valid());
//! ```

use crate::ContextMap;
use std::sync::Arc;

use crate::component::Context;

/// A single validation rule for a [`FormControl`]'s current value.
/// A regra de um [`Validator::Custom`]: recebe o valor do campo e devolve
/// `Ok(())` ou a mensagem de erro. A `String` aqui é **de propósito** — é o texto
/// que o usuário final vê no formulário, não um erro do motor (que seria um
/// [`crate::GlacierError`]).
pub type CustomRule = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

#[derive(Clone)]
pub enum Validator {
    /// The value must not be empty (after trimming whitespace).
    Required,
    /// The value must have at least this many characters.
    MinLength(usize),
    /// The value must have at most this many characters.
    MaxLength(usize),
    /// After stripping every non-digit, the value must have between `min` and
    /// `max` digits (inclusive). `digits:11` sets both to 11; `digits:10,11`
    /// spans a range. Built for masked inputs (CPF, phone) whose stored value
    /// keeps no punctuation.
    Digits { min: usize, max: usize },
    /// The value, parsed as a number, must be `>=` this. Non-numeric fails.
    Gte(f64),
    /// The value, parsed as a number, must be `<=` this. Non-numeric fails.
    Lte(f64),
    /// The value must read as accepted — `"true"`, `"on"`, `"1"`, `"yes"`,
    /// `"sim"` (case-insensitive). For a `<checkbox>`'s `"true"`/`"false"`.
    Accepted,
    /// The value must look like an email address (a pragmatic pattern, not the
    /// full RFC). Empty passes — pair with `Required` to forbid blank.
    Email,
    /// The value must match this regular expression (a value only fails if
    /// the pattern is well-formed and does *not* match — a malformed pattern
    /// is reported as its own error instead of silently passing).
    Pattern(String),
    /// Any other rule: `Ok(())` when valid, `Err(message)` otherwise.
    Custom(CustomRule),
    /// A rule resolved outside this module: the *name* of a handler function
    /// the host calls with the value (`fn:validar_cpf` in a `rules` string).
    /// [`Validator::check`] skips it — the caller that owns the handler runs
    /// it and folds the result into the control's errors.
    Script(String),
}

impl std::fmt::Debug for Validator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Validator::Required => write!(f, "Required"),
            Validator::MinLength(n) => write!(f, "MinLength({n})"),
            Validator::MaxLength(n) => write!(f, "MaxLength({n})"),
            Validator::Digits { min, max } => write!(f, "Digits({min}..={max})"),
            Validator::Gte(n) => write!(f, "Gte({n})"),
            Validator::Lte(n) => write!(f, "Lte({n})"),
            Validator::Accepted => write!(f, "Accepted"),
            Validator::Email => write!(f, "Email"),
            Validator::Pattern(p) => write!(f, "Pattern({p:?})"),
            Validator::Custom(_) => write!(f, "Custom(..)"),
            Validator::Script(name) => write!(f, "Script({name:?})"),
        }
    }
}

impl Validator {
    /// Checks `value` against this rule, returning a user-facing error
    /// message naming `field` when it fails.
    fn check(&self, field: &str, value: &str) -> Result<(), String> {
        match self {
            Validator::Required => {
                if value.trim().is_empty() {
                    Err(format!("\"{field}\" is required"))
                } else {
                    Ok(())
                }
            }
            Validator::MinLength(n) => {
                if value.chars().count() < *n {
                    Err(format!("\"{field}\" must be at least {n} characters"))
                } else {
                    Ok(())
                }
            }
            Validator::MaxLength(n) => {
                if value.chars().count() > *n {
                    Err(format!("\"{field}\" must be at most {n} characters"))
                } else {
                    Ok(())
                }
            }
            Validator::Digits { min, max } => {
                let n = value.chars().filter(char::is_ascii_digit).count();
                if n < *min || n > *max {
                    if min == max {
                        Err(format!("\"{field}\" must have {min} digits"))
                    } else {
                        Err(format!("\"{field}\" must have between {min} and {max} digits"))
                    }
                } else {
                    Ok(())
                }
            }
            Validator::Gte(limit) => match value.trim().parse::<f64>() {
                Ok(n) if n >= *limit => Ok(()),
                _ => Err(format!("\"{field}\" must be at least {limit}")),
            },
            Validator::Lte(limit) => match value.trim().parse::<f64>() {
                Ok(n) if n <= *limit => Ok(()),
                _ => Err(format!("\"{field}\" must be at most {limit}")),
            },
            Validator::Accepted => {
                let v = value.trim().to_ascii_lowercase();
                if matches!(v.as_str(), "true" | "on" | "1" | "yes" | "sim") {
                    Ok(())
                } else {
                    Err(format!("\"{field}\" must be accepted"))
                }
            }
            Validator::Email => {
                if value.trim().is_empty() {
                    return Ok(());
                }
                // Pragmatic: one @, non-empty local part, a dot in the domain,
                // no whitespace. Not the RFC — enough to catch a typo.
                match regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$") {
                    Ok(re) if re.is_match(value.trim()) => Ok(()),
                    _ => Err(format!("\"{field}\" is not a valid email")),
                }
            }
            Validator::Script(_) => Ok(()),
            Validator::Pattern(pattern) => match regex::Regex::new(pattern) {
                Ok(re) => {
                    if re.is_match(value) {
                        Ok(())
                    } else {
                        Err(format!("\"{field}\" does not match the expected format"))
                    }
                }
                Err(e) => Err(format!("\"{field}\" has an invalid pattern: {e}")),
            },
            Validator::Custom(f) => f(value),
        }
    }

    /// Parses a `rules` attribute — `"required|minlen:3|digits:11"` — into
    /// validators in declaration order. Tokens are `|`-separated; an argument
    /// follows a `:`. `fn:NAME` becomes [`Validator::Script`]. The first
    /// malformed token is returned as `Err` (surfaced as a template error, not
    /// swallowed). `pattern` is *not* a token here — it stays a distinct
    /// attribute, since a regex freely contains `|` and `:`.
    pub fn parse_rules(spec: &str) -> Result<Vec<Validator>, String> {
        fn need<'a>(name: &str, a: Option<&'a str>) -> Result<&'a str, String> {
            a.filter(|s| !s.is_empty())
                .ok_or_else(|| format!("rule `{name}` needs an argument"))
        }
        fn whole(name: &str, a: Option<&str>) -> Result<usize, String> {
            let s = need(name, a)?;
            s.parse::<usize>()
                .map_err(|_| format!("rule `{name}`: `{s}` is not a whole number"))
        }
        fn real(name: &str, a: Option<&str>) -> Result<f64, String> {
            let s = need(name, a)?;
            s.parse::<f64>()
                .map_err(|_| format!("rule `{name}`: `{s}` is not a number"))
        }

        let mut out = Vec::new();
        for raw in spec.split('|') {
            let tok = raw.trim();
            if tok.is_empty() {
                continue;
            }
            let (name, arg) = match tok.split_once(':') {
                Some((n, a)) => (n.trim(), Some(a.trim())),
                None => (tok, None),
            };
            let v = match name {
                "required" | "req" => Validator::Required,
                "accepted" | "accept" => Validator::Accepted,
                "email" => Validator::Email,
                "minlen" | "minlength" => Validator::MinLength(whole(name, arg)?),
                "maxlen" | "maxlength" => Validator::MaxLength(whole(name, arg)?),
                "gte" => Validator::Gte(real(name, arg)?),
                "lte" => Validator::Lte(real(name, arg)?),
                "digits" => {
                    let a = need(name, arg)?;
                    let (lo, hi) = match a.split_once(',') {
                        Some((l, r)) => (
                            whole("digits", Some(l.trim())),
                            whole("digits", Some(r.trim())),
                        ),
                        None => (whole("digits", Some(a)), whole("digits", Some(a))),
                    };
                    Validator::Digits {
                        min: lo?,
                        max: hi?,
                    }
                }
                "fn" => Validator::Script(need(name, arg)?.to_string()),
                other => return Err(format!("unknown validation rule `{other}`")),
            };
            out.push(v);
        }
        Ok(out)
    }
}

/// A single field of a [`Form`]: a name, a current value, and the validators
/// that value must satisfy. Built via [`FormControl::new`] and the builder
/// methods ([`FormControl::required`], etc.), then added to a [`FormBuilder`].
#[derive(Clone, Debug)]
pub struct FormControl {
    name: String,
    value: String,
    initial_value: String,
    validators: Vec<Validator>,
    touched: bool,
    errors: Vec<String>,
}

impl FormControl {
    /// A new control named `name`, seeded with `initial_value` and no
    /// validators (add them with `.required()`/`.min_length()`/etc.).
    pub fn new(name: impl Into<String>, initial_value: impl Into<String>) -> Self {
        let value = initial_value.into();
        Self {
            name: name.into(),
            value: value.clone(),
            initial_value: value,
            validators: Vec::new(),
            touched: false,
            errors: Vec::new(),
        }
    }

    /// The value must not be empty (after trimming whitespace).
    pub fn required(mut self) -> Self {
        self.validators.push(Validator::Required);
        self
    }

    /// The value must have at least `n` characters.
    pub fn min_length(mut self, n: usize) -> Self {
        self.validators.push(Validator::MinLength(n));
        self
    }

    /// The value must have at most `n` characters.
    pub fn max_length(mut self, n: usize) -> Self {
        self.validators.push(Validator::MaxLength(n));
        self
    }

    /// The value must match the regular expression `pattern`.
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.validators.push(Validator::Pattern(pattern.into()));
        self
    }

    /// After stripping non-digits, between `min` and `max` digits (inclusive).
    pub fn digits(mut self, min: usize, max: usize) -> Self {
        self.validators.push(Validator::Digits { min, max });
        self
    }

    /// The value, parsed as a number, must be `>= n`.
    pub fn gte(mut self, n: f64) -> Self {
        self.validators.push(Validator::Gte(n));
        self
    }

    /// The value, parsed as a number, must be `<= n`.
    pub fn lte(mut self, n: f64) -> Self {
        self.validators.push(Validator::Lte(n));
        self
    }

    /// The value must read as accepted (`"true"`/`"on"`/`"1"`/`"yes"`/`"sim"`).
    pub fn accepted(mut self) -> Self {
        self.validators.push(Validator::Accepted);
        self
    }

    /// The value must look like an email address (empty passes).
    pub fn email(mut self) -> Self {
        self.validators.push(Validator::Email);
        self
    }

    /// A control pre-loaded with `validators` — e.g. the output of
    /// [`Validator::parse_rules`] applied to a `<input rules="...">` string.
    pub fn with_validators(
        name: impl Into<String>,
        initial_value: impl Into<String>,
        validators: Vec<Validator>,
    ) -> Self {
        let mut c = Self::new(name, initial_value);
        c.validators = validators;
        c
    }

    /// The names of every [`Validator::Script`] rule on this control, in
    /// declaration order — the host resolves these itself (see
    /// [`Validator::Script`]).
    pub fn script_rules(&self) -> impl Iterator<Item = &str> {
        self.validators.iter().filter_map(|v| match v {
            Validator::Script(n) => Some(n.as_str()),
            _ => None,
        })
    }

    /// Appends a message to the cached errors — for a host-side rule
    /// ([`Validator::Script`]) folding its result in after [`Form::validate`].
    pub fn push_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }

    /// Any other rule: `f` returns `Ok(())` when `value` is valid, or
    /// `Err(message)` with a user-facing explanation otherwise.
    pub fn validator<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        self.validators.push(Validator::Custom(Arc::new(f)));
        self
    }

    /// This control's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// This control's current value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Whether the value has ever been changed via [`FormControl::set_value`]
    /// (or the control force-validated via [`Form::validate`]).
    pub fn touched(&self) -> bool {
        self.touched
    }

    /// The errors from the last validation, in validator declaration order.
    /// Populated by [`FormControl::set_value`] and [`Form::validate`]; use
    /// [`FormControl::is_valid`] for a check that is always fresh.
    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    /// Updates the value, marks the control touched, and re-runs its
    /// validators (see [`FormControl::errors`]).
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.touched = true;
        self.errors = self.collect_errors();
    }

    /// Restores the initial value and clears touched/errors.
    pub fn reset(&mut self) {
        self.value = self.initial_value.clone();
        self.touched = false;
        self.errors.clear();
    }

    /// Runs every validator against the current value fresh (independent of
    /// the cached [`FormControl::errors`]) and reports whether all passed.
    pub fn is_valid(&self) -> bool {
        self.collect_errors().is_empty()
    }

    fn collect_errors(&self) -> Vec<String> {
        self.validators
            .iter()
            .filter_map(|v| v.check(&self.name, &self.value).err())
            .collect()
    }
}

/// Declares a [`Form`]'s controls, in the order inputs should be visited when
/// the user presses Enter to move to the next field.
///
/// ```
/// use glacier_ui::{FormBuilder, FormControl};
///
/// let form = FormBuilder::new("signup")
///     .control(FormControl::new("email", "").required().pattern(r"^[^@\s]+@[^@\s]+\.[^@\s]+$"))
///     .control(FormControl::new("password", "").required().min_length(6))
///     .build();
/// assert_eq!(form.control_names().collect::<Vec<_>>(), vec!["email", "password"]);
/// ```
/// A `Form`'s registered submit handler (see [`FormBuilder::on_submit`] /
/// [`Form::submit`]). Takes the form itself (so it can check
/// [`Form::is_valid`], read values, mark errors via [`Form::validate`], ...)
/// and the reactive [`Context`] (to publish a result, navigate, etc).
type SubmitHandler = dyn Fn(&mut Form, &mut Context<'_>) + Send + Sync;

pub struct FormBuilder {
    name: String,
    controls: Vec<FormControl>,
    on_submit: Option<Arc<SubmitHandler>>,
}

impl FormBuilder {
    /// Starts building a form named `name` (matched against a `<Form
    /// name="...">`'s `name` attribute when more than one form shares a
    /// component; otherwise it can be any label).
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            controls: Vec::new(),
            on_submit: None,
        }
    }

    /// Registers a control, in the order controls should be visited by Enter.
    pub fn control(mut self, control: FormControl) -> Self {
        self.controls.push(control);
        self
    }

    /// Registers the closure [`Form::submit`] runs. Keeps the submit logic
    /// declared right next to the form's controls, instead of spread across
    /// a `Component::on_form_submit`/`update` match:
    /// ```ignore
    /// let form = FormBuilder::new("login")
    ///     .control(FormControl::new("username", "").required())
    ///     .on_submit(|form, ctx| {
    ///         if form.is_valid() {
    ///             ctx.set("status", "ok");
    ///         } else {
    ///             form.validate();
    ///         }
    ///     })
    ///     .build();
    /// ```
    /// Called from `Component::on_form_submit`:
    /// ```ignore
    /// fn on_form_submit(&mut self, _action: &str, ctx: &mut Context) {
    ///     self.form.submit(ctx);
    /// }
    /// ```
    pub fn on_submit<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut Form, &mut Context<'_>) + Send + Sync + 'static,
    {
        self.on_submit = Some(Arc::new(f));
        self
    }

    /// Finishes building the [`Form`].
    pub fn build(self) -> Form {
        Form {
            name: self.name,
            controls: self.controls,
            on_submit: self.on_submit,
        }
    }
}

/// A group of [`FormControl`]s built by [`FormBuilder`]. Owned by the
/// component's own state (not the engine), synced into the reactive
/// [`Context`] via [`Form::sync_to_context`] so `formControl`-bound inputs
/// read/write it.
pub struct Form {
    name: String,
    controls: Vec<FormControl>,
    on_submit: Option<Arc<SubmitHandler>>,
}

impl Form {
    /// This form's name (see [`FormBuilder::new`]).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The control named `name`, if any.
    pub fn get(&self, name: &str) -> Option<&FormControl> {
        self.controls.iter().find(|c| c.name == name)
    }

    /// A mutable reference to the control named `name`, if any.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut FormControl> {
        self.controls.iter_mut().find(|c| c.name == name)
    }

    /// Whether a control named `name` is registered on this form. Handy in
    /// `Component::update` to dispatch a form-control action generically:
    /// ```ignore
    /// if self.form.has_control(action) {
    ///     self.form.set_value(action, value.unwrap_or_default());
    ///     self.form.sync_to_context(ctx);
    /// }
    /// ```
    pub fn has_control(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// The current value of `name` (`""` if there's no such control).
    pub fn value(&self, name: &str) -> &str {
        self.get(name).map(FormControl::value).unwrap_or("")
    }

    /// Updates the value of the control named `name`, if it exists (a no-op
    /// otherwise).
    pub fn set_value(&mut self, name: &str, value: impl Into<String>) {
        if let Some(c) = self.get_mut(name) {
            c.set_value(value);
        }
    }

    /// The cached errors of `name` (`&[]` if there's no such control, or it
    /// hasn't been validated yet).
    pub fn errors(&self, name: &str) -> &[String] {
        self.get(name).map(FormControl::errors).unwrap_or(&[])
    }

    /// Publishes every control's first cached error (`""` if none) into the
    /// context under `"{prefix}{name}"` (e.g. `prefix="erro_"` ->
    /// `"erro_username"`), for a `Text "{erro_username}"` placeholder to show
    /// inline. Reads the same cache as [`Form::errors`] — populated by
    /// [`FormControl::set_value`]/[`Form::validate`], so it stays blank until
    /// the user has actually interacted, instead of showing "required" before
    /// they've typed anything.
    pub fn errors_to_context(&self, ctx: &mut Context, prefix: &str) {
        for c in &self.controls {
            ctx.set(
                &format!("{prefix}{}", c.name),
                c.errors.first().cloned().unwrap_or_default(),
            );
        }
    }

    /// Whether every control currently passes its validators. Always fresh —
    /// safe to call before the user has touched anything (e.g. a submit
    /// button that starts disabled).
    pub fn is_valid(&self) -> bool {
        self.controls.iter().all(FormControl::is_valid)
    }

    /// Force-runs every control's validators, marking all of them touched and
    /// caching the resulting errors — so a submit handler can surface errors
    /// on fields the user never edited. Returns the same as
    /// [`Form::is_valid`].
    pub fn validate(&mut self) -> bool {
        let mut all_valid = true;
        for c in self.controls.iter_mut() {
            c.touched = true;
            c.errors = c.collect_errors();
            if !c.errors.is_empty() {
                all_valid = false;
            }
        }
        all_valid
    }

    /// Restores every control to its initial value and clears touched/errors.
    pub fn reset(&mut self) {
        for c in self.controls.iter_mut() {
            c.reset();
        }
    }

    /// Every control's name, in declaration order.
    pub fn control_names(&self) -> impl Iterator<Item = &str> {
        self.controls.iter().map(FormControl::name)
    }

    /// A snapshot of every control's current value, keyed by name.
    pub fn values(&self) -> ContextMap {
        self.controls
            .iter()
            .map(|c| (c.name.clone(), c.value.clone()))
            .collect()
    }

    /// Publishes every control's current value into the reactive context under
    /// its own name, so `formControl`-bound inputs (and any `{name}`
    /// placeholder) reflect the form's state. Call this from
    /// `Component::init`/`update` after any change made directly through the
    /// `Form` API (a change coming from the input itself round-trips through
    /// `Form::set_value` already).
    pub fn sync_to_context(&self, ctx: &mut Context) {
        for c in &self.controls {
            ctx.set(&c.name, c.value.clone());
        }
    }

    /// Runs the closure registered via [`FormBuilder::on_submit`], if any (a
    /// no-op otherwise). Typically the entire body of
    /// `Component::on_form_submit`:
    /// ```ignore
    /// fn on_form_submit(&mut self, _action: &str, ctx: &mut Context) {
    ///     self.form.submit(ctx);
    /// }
    /// ```
    pub fn submit(&mut self, ctx: &mut Context) {
        // Taken out for the call so the closure can take `&mut Form` (i.e.
        // `self`) without also holding a borrow of the very field it's
        // stored in; put back after, since it's an `Arc` (cheap to move).
        if let Some(handler) = self.on_submit.take() {
            handler(self, ctx);
            self.on_submit = Some(handler);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn required_control_is_invalid_until_set() {
        let form = FormBuilder::new("f")
            .control(FormControl::new("username", ""))
            .build();
        // No validators yet: an empty value is still "valid".
        assert!(form.is_valid());

        let form = FormBuilder::new("f")
            .control(FormControl::new("username", "").required())
            .build();
        assert!(!form.is_valid());
    }

    #[test]
    fn set_value_updates_touched_and_errors_live() {
        let mut form = FormBuilder::new("f")
            .control(FormControl::new("username", "").required().min_length(3))
            .build();

        assert!(!form.get("username").unwrap().touched());
        assert!(!form.is_valid());

        form.set_value("username", "ab");
        assert!(form.get("username").unwrap().touched());
        assert!(!form.is_valid());
        assert_eq!(form.errors("username").len(), 1);

        form.set_value("username", "abc");
        assert!(form.is_valid());
        assert!(form.errors("username").is_empty());
    }

    #[test]
    fn min_and_max_length() {
        let mut form = FormBuilder::new("f")
            .control(FormControl::new("bio", "").min_length(2).max_length(4))
            .build();

        form.set_value("bio", "a");
        assert!(!form.is_valid());
        form.set_value("bio", "ab");
        assert!(form.is_valid());
        form.set_value("bio", "abcd");
        assert!(form.is_valid());
        form.set_value("bio", "abcde");
        assert!(!form.is_valid());
    }

    #[test]
    fn pattern_validator() {
        let mut form = FormBuilder::new("f")
            .control(FormControl::new("email", "").pattern(r"^[^@\s]+@[^@\s]+\.[^@\s]+$"))
            .build();

        form.set_value("email", "not-an-email");
        assert!(!form.is_valid());
        form.set_value("email", "user@example.com");
        assert!(form.is_valid());
    }

    #[test]
    fn custom_validator_closure() {
        let mut form = FormBuilder::new("f")
            .control(FormControl::new("age", "0").validator(|v| {
                v.parse::<u32>()
                    .ok()
                    .filter(|n| *n >= 18)
                    .map(|_| ())
                    .ok_or_else(|| "must be an adult".to_string())
            }))
            .build();

        assert!(!form.is_valid());
        form.set_value("age", "17");
        assert!(!form.is_valid());
        assert_eq!(form.errors("age"), &["must be an adult".to_string()]);
        form.set_value("age", "18");
        assert!(form.is_valid());
    }

    #[test]
    fn validate_marks_every_control_touched_even_if_untouched() {
        let mut form = FormBuilder::new("f")
            .control(FormControl::new("a", "").required())
            .control(FormControl::new("b", "ok"))
            .build();

        assert!(!form.get("a").unwrap().touched());
        let ok = form.validate();
        assert!(!ok);
        assert!(form.get("a").unwrap().touched());
        assert!(form.get("b").unwrap().touched());
        assert_eq!(form.errors("a").len(), 1);
    }

    #[test]
    fn parse_rules_reads_the_dsl() {
        let vs = Validator::parse_rules("required|minlen:3|digits:10,11|gte:18|accepted|fn:cpf")
            .unwrap();
        assert!(matches!(vs[0], Validator::Required));
        assert!(matches!(vs[1], Validator::MinLength(3)));
        assert!(matches!(vs[2], Validator::Digits { min: 10, max: 11 }));
        assert!(matches!(vs[3], Validator::Gte(n) if n == 18.0));
        assert!(matches!(vs[4], Validator::Accepted));
        assert!(matches!(&vs[5], Validator::Script(n) if n == "cpf"));

        assert!(Validator::parse_rules("required|bogus:1").is_err());
        assert!(Validator::parse_rules("minlen").is_err());
        assert!(Validator::parse_rules("digits:abc").is_err());
    }

    #[test]
    fn digits_ignores_punctuation() {
        let mut form = FormBuilder::new("f")
            .control(FormControl::with_validators(
                "cpf",
                "",
                Validator::parse_rules("digits:11").unwrap(),
            ))
            .build();
        form.set_value("cpf", "123.456.789-0");
        assert!(!form.is_valid());
        form.set_value("cpf", "123.456.789-01");
        assert!(form.is_valid());
    }

    #[test]
    fn script_rule_is_skipped_by_check_but_listed() {
        let c = FormControl::with_validators(
            "cpf",
            "x",
            Validator::parse_rules("required|fn:validar_cpf").unwrap(),
        );
        assert_eq!(c.script_rules().collect::<Vec<_>>(), vec!["validar_cpf"]);
        // `check` (via is_valid) treats Script as a pass — the host runs it.
        assert!(c.is_valid());
    }

    #[test]
    fn reset_restores_initial_value() {
        let mut form = FormBuilder::new("f")
            .control(FormControl::new("name", "Ana"))
            .build();

        form.set_value("name", "Bob");
        assert_eq!(form.value("name"), "Bob");
        form.reset();
        assert_eq!(form.value("name"), "Ana");
        assert!(!form.get("name").unwrap().touched());
    }

    #[test]
    fn values_and_control_names_preserve_declaration_order() {
        let form = FormBuilder::new("f")
            .control(FormControl::new("first", "1"))
            .control(FormControl::new("second", "2"))
            .build();

        assert_eq!(
            form.control_names().collect::<Vec<_>>(),
            vec!["first", "second"]
        );
        let values = form.values();
        assert_eq!(values.get("first").map(String::as_str), Some("1"));
        assert_eq!(values.get("second").map(String::as_str), Some("2"));
    }

    #[test]
    fn has_control_and_missing_control_are_harmless() {
        let mut form = FormBuilder::new("f")
            .control(FormControl::new("x", ""))
            .build();

        assert!(form.has_control("x"));
        assert!(!form.has_control("y"));
        // Setting a value on a control that doesn't exist is a no-op, not a panic.
        form.set_value("y", "z");
        assert_eq!(form.value("y"), "");
        assert!(form.errors("y").is_empty());
    }

    fn test_context(data: &mut ContextMap) -> Context<'_> {
        Context::new(data)
    }

    #[test]
    fn submit_runs_the_registered_closure() {
        let mut form = FormBuilder::new("f")
            .control(FormControl::new("username", "").required())
            .on_submit(|form, ctx| {
                if form.is_valid() {
                    ctx.set("status", "ok");
                } else {
                    form.validate();
                    ctx.set("status", "invalid");
                }
            })
            .build();

        let mut data = HashMap::default();
        form.submit(&mut test_context(&mut data));
        assert_eq!(data.get("status").map(String::as_str), Some("invalid"));
        // `validate()` inside the closure ran against the real `form`, not a copy.
        assert!(form.get("username").unwrap().touched());

        form.set_value("username", "ana");
        let mut data = HashMap::default();
        form.submit(&mut test_context(&mut data));
        assert_eq!(data.get("status").map(String::as_str), Some("ok"));
    }

    #[test]
    fn submit_without_a_registered_closure_is_a_no_op() {
        let mut form = FormBuilder::new("f")
            .control(FormControl::new("x", ""))
            .build();
        let mut data = HashMap::default();
        form.submit(&mut test_context(&mut data)); // must not panic
        assert!(data.is_empty());
    }

    #[test]
    fn errors_to_context_publishes_first_cached_error_per_control() {
        let mut form = FormBuilder::new("f")
            .control(FormControl::new("username", "").required())
            .control(FormControl::new("bio", ""))
            .build();

        let mut data = HashMap::default();
        // Untouched: blank, not "required" — errors are cached, not fresh.
        form.errors_to_context(&mut test_context(&mut data), "erro_");
        assert_eq!(data.get("erro_username").map(String::as_str), Some(""));
        assert_eq!(data.get("erro_bio").map(String::as_str), Some(""));

        form.set_value("username", "");
        let mut data = HashMap::default();
        form.errors_to_context(&mut test_context(&mut data), "erro_");
        assert_eq!(
            data.get("erro_username").map(String::as_str),
            Some("\"username\" is required")
        );
    }
}
