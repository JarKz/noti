use shared::unique::Unique;

use crate::{
    stage::measure::{Constraints, Intrinsic},
    state::{MutableState, State, StateInfo},
    types::{
        dirty_flags::DirtyFlags, identifiers::WidgetKey, Extent, StyleInfo, WidgetClass, WidgetId,
        WidgetStyle,
    },
};
use std::{any::Any, collections::HashMap, time::Duration};

/// A context for a widget tree that stores runtime data and other information associated with its
/// widgets.
///
/// It is essential to a widget tree because widgets are blueprints, while their runtime data is
/// stored in the context. The context can operate on widgets without depending on their structure
/// and provides the information required to perform runtime operations.
///
/// The implementation of [Context] is hidden from users but available to developers.
/// See the internal documentation for more details.
pub struct Context {
    debug_options: DebugOptions,

    /// The id counter for generating identifiers for widgets.
    id_counter: u64,

    /// The number counter for generating state descriptors.
    state_descriptor_counter: usize,

    /// Registry of arbitrary data associated with particular widgets.
    widget_data_registry: HashMap<WidgetId, Box<dyn Any>>,

    /// Registry of measurement caches for avoiding unnecessary widget re-measurement.
    measure_cache: HashMap<WidgetId, MeasureCache>,

    /// Registry of mutable and immutable states bound to their descriptors.
    state_registry: HashMap<usize, StateInfo>,

    /// Registry of unique widget keys mapped to known widget IDs.
    key_registry: HashMap<WidgetKey, WidgetId>,

    /// Registry of widget style information bound to widget classes.
    style_registry: HashMap<WidgetClass, StyleInfo>,

    /// Registry of dirty flags bound to widget IDs.
    dirty_registry: HashMap<WidgetId, DirtyFlags>,

    /// Registry of animation progress associated with widget IDs.
    animation_regirsty: HashMap<WidgetId, AnimationProgress>,

    /// Global storage of fonts used for text rendering without wasting memory.
    font_collection: skia_safe::textlayout::FontCollection,
}

impl Context {
    pub fn new(font_collection: skia_safe::textlayout::FontCollection) -> Self {
        Self {
            debug_options: DebugOptions::default(),
            id_counter: 1,
            state_descriptor_counter: 1,
            font_collection,
            widget_data_registry: HashMap::new(),
            measure_cache: HashMap::new(),
            state_registry: HashMap::new(),
            key_registry: HashMap::new(),
            style_registry: HashMap::new(),
            dirty_registry: HashMap::new(),
            animation_regirsty: HashMap::new(),
        }
    }

    pub fn update_debug_options(&mut self, debug_options: DebugOptions) {
        self.debug_options = debug_options;
    }

    fn register_state(&mut self, state_info: StateInfo) -> usize {
        let descriptor = self.state_descriptor_counter;
        self.state_descriptor_counter += 1;
        self.state_registry.insert(descriptor, state_info);
        descriptor
    }

    pub(super) fn is_invalidation_required(&self) -> bool {
        !self.dirty_registry.is_empty()
    }
}

/// A collection of options that made for debuggin an UI or internal work detail.
#[derive(Clone, Debug)]
pub struct DebugOptions {
    /// Shows the layout bounds of each UI element using different colors.
    pub show_layout_bounds: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            show_layout_bounds: false,
        }
    }
}

/// A collection of a widget measurement. Usually these widget measurements
/// are cached into [Context] to reuse them further witouth re-measuring.
#[derive(Debug, Clone, Default)]
struct MeasureCache {
    /// The intrinsic size of a widget.
    ///
    /// A widget must know what is the minimal and the maximal size of itself.
    intrinsic: Option<Intrinsic<f32>>,

    /// The actual exetnt of a widget under a speficif constraints.
    extent: Option<Extent<f32>>,

    /// The constraints that used to a particular widget. It uses for re-measuring to check whether
    /// constraints were changed or not.
    ///
    /// The rule is simple: if constraints were changed then do re-measurement, otherwise use the
    /// cached extent.
    constraints: Option<Constraints<Extent<f32>>>,
}

/// The information about animation for a widget.
///
/// The animation progress is detached from widget, but provides information to it. It allows tick a
/// progress independently and tell the [Context] whether to call a widget to rebuild or not.
#[derive(Debug, Clone)]
pub(crate) struct AnimationProgress {
    /// The time was passed since start in nanoseconds.
    ///
    /// Actual start might be at zero or at maximum duration depending on an animation direction. If
    /// it passing forward then it will start at zero, otherwise at maximum duration but moves
    /// toward to zero.
    elapsed_ns: u128,

    /// The maximum duration of an animation.
    duration: Duration,

    /// The dirty flags to pass a widget to trigger some actions from the [Context].
    flag_on_change: DirtyFlags,

    /// The animation direction telling to increase or decrease the passed time.
    direction: AnimationDirection,
}

#[derive(Debug, Clone)]
pub(crate) enum AnimationDirection {
    Forward,
    Backward,
}

impl AnimationProgress {
    pub(crate) fn new(
        duration: Duration,
        flag_on_change: DirtyFlags,
        direction: AnimationDirection,
    ) -> Self {
        Self {
            elapsed_ns: match &direction {
                AnimationDirection::Forward => 0,
                AnimationDirection::Backward => duration.as_nanos(),
            },
            duration,
            flag_on_change,
            direction,
        }
    }

    fn reverse(&mut self) {
        self.direction.reverse()
    }

    fn progress(&self) -> f32 {
        self.elapsed_ns as f32 / self.duration.as_nanos() as f32
    }

    fn is_finished(&self) -> bool {
        match self.direction {
            AnimationDirection::Forward => self.elapsed_ns == self.duration.as_nanos(),
            AnimationDirection::Backward => self.elapsed_ns == 0,
        }
    }
}

impl AnimationDirection {
    fn reverse(&mut self) {
        *self = match self {
            AnimationDirection::Forward => AnimationDirection::Backward,
            AnimationDirection::Backward => AnimationDirection::Forward,
        }
    }
}

impl Tick for AnimationProgress {
    fn tick(&mut self, delta_ns: u128) {
        match self.direction {
            AnimationDirection::Forward => {
                let duration_ns = self.duration.as_nanos();
                self.elapsed_ns = (self.elapsed_ns + delta_ns).min(duration_ns);
            }
            AnimationDirection::Backward => {
                self.elapsed_ns = self.elapsed_ns - delta_ns.min(self.elapsed_ns);
            }
        }
    }
}

pub trait GetDebugOptions {
    /// Retrieves debug options.
    fn get_debug_options(&self) -> &DebugOptions;
}

impl GetDebugOptions for Context {
    fn get_debug_options(&self) -> &DebugOptions {
        &self.debug_options
    }
}

/// Retrieves a reference to a widget data that bound to its id.
///
/// Instead of trying to write the whole turbofish generics better to use output matching:
///```rs
///let some_data: &WidgetData = widget_data(context, &self.id).except("There's must be a widget data.")
///```
pub fn widget_data<T: 'static, C, Id>(context: &C, widget_id: Id) -> Option<&T>
where
    C: ManageWidgetData<Id>,
    Id: Into<WidgetId>,
{
    context
        .get_widget_data(widget_id)
        .and_then(|data| data.downcast_ref::<T>())
}

/// Retrieves an unique access to a widget data that bound to its id.
///
/// Instead of trying to write the whole turbofish generics better to use output matching:
///```rs
///let some_data: Unique<WidgetData> = widget_data_mut(context, &self.id).except("There's must be a widget data.")
///```
pub fn widget_data_mut<T: 'static, C, Id>(context: &mut C, widget_id: Id) -> Option<Unique<T>>
where
    C: ManageWidgetData<Id>,
    Id: Into<WidgetId>,
{
    context
        .get_widget_data_mut(widget_id)
        .and_then(|data| data.downcast_mut::<T>())
        .map(|data| unsafe { Unique::from_mut(data) })
}

pub trait ManageWidgetData<Id>
where
    Id: Into<WidgetId>,
{
    /// Saves a provided widget data by its id into a context.
    fn set_widget_data(&mut self, widget_id: Id, widget_data: Box<dyn Any>);

    /// Retrieves an associated widget data by reference.
    fn get_widget_data(&self, widget_id: Id) -> Option<&dyn Any>;

    /// Retrieves an associated widget data by mutable reference.
    fn get_widget_data_mut(&mut self, widget_id: Id) -> Option<&mut dyn Any>;

    /// Removes a widget data from a context.
    fn remove_widget_data(&mut self, widget_id: Id);
}

impl<Id> ManageWidgetData<Id> for Context
where
    Id: Into<WidgetId>,
{
    fn set_widget_data(&mut self, widget_id: Id, widget_data: Box<dyn Any>) {
        self.widget_data_registry
            .insert(widget_id.into(), widget_data);
    }

    fn get_widget_data(&self, widget_id: Id) -> Option<&dyn Any> {
        self.widget_data_registry
            .get(&widget_id.into())
            .map(|data| &**data)
    }
    fn get_widget_data_mut(&mut self, widget_id: Id) -> Option<&mut dyn Any> {
        self.widget_data_registry
            .get_mut(&widget_id.into())
            .map(|data| &mut **data)
    }

    fn remove_widget_data(&mut self, widget_id: Id) {
        self.widget_data_registry.remove(&widget_id.into());
    }
}

pub trait CreateState<T> {
    /// Creates a state with a provided value in a context.
    fn create_state(&mut self, value: T) -> State<T>;

    /// Creates a mutable stata with a provided value in a context.
    fn create_state_mut(&mut self, value: T) -> MutableState<T>;
}

impl<T> CreateState<T> for Context {
    fn create_state(&mut self, value: T) -> State<T> {
        let state_info = StateInfo::new(value, false);
        State::new(self.register_state(state_info))
    }

    fn create_state_mut(&mut self, value: T) -> MutableState<T> {
        let state_info = StateInfo::new(value, true);
        MutableState::new(self.register_state(state_info))
    }
}

/// The scoped context that solves the [Context] issues with traits that cannot be dyn-compatible.
///
/// It aims to scope users callbacks, in which enough to manage states. And with it the [Context]
/// can be reduced to it's capabilities of [GetState] and [SetState] by using dyn-compatible [ScopedManageState].
pub struct ScopedContext<'a> {
    inner: &'a mut dyn ScopedManageState,
}

impl<'a> ScopedContext<'a> {
    pub(crate) fn new<C: ScopedManageState>(context: &'a mut C) -> Self {
        Self { inner: context }
    }
}

/// The dyn-compatible version of [GetState] and [SetState].
pub(crate) trait ScopedManageState {
    fn get_raw(&self, descriptor: usize) -> Option<&dyn Any>;
    fn set_raw(&mut self, descriptor: usize, val: Box<dyn Any>);
}

impl ScopedManageState for Context {
    fn set_raw(&mut self, descriptor: usize, val: Box<dyn Any>) {
        if let Some(state_info) = self.state_registry.get_mut(&descriptor) {
            if state_info.set_raw_data(val) {
                // TODO: check the validity of exising widget
                // For instance, in subscriber list may be some unexisting widget and marking dirty
                // flags will be invalid
                state_info.subscribers().for_each(|subscriber| {
                    self.dirty_registry
                        .entry(*subscriber)
                        .and_modify(|flags| *flags |= DirtyFlags::NEEDS_REBUILD)
                        .or_insert(DirtyFlags::NEEDS_REBUILD);
                });
            }
        }
    }

    fn get_raw(&self, descriptor: usize) -> Option<&dyn Any> {
        self.state_registry
            .get(&descriptor)
            .and_then(|state_info| state_info.get_raw_data())
    }
}

pub trait GetState {
    /// Retrieves a state by it's descriptor.
    fn get<T: 'static, S: Into<State<T>>>(&self, state: S) -> Option<&T>;
}

pub trait SetState {
    /// Assigns a value to a state by it's descriptor.
    fn set<T: 'static>(&mut self, mutable_state: MutableState<T>, value: T);
}

impl GetState for Context {
    fn get<T, S>(&self, state: S) -> Option<&T>
    where
        T: 'static,
        S: Into<State<T>>,
    {
        let state = state.into();
        self.state_registry
            .get(&state.descriptor)
            .and_then(|state_info| state_info.get_data(state))
    }
}

impl SetState for Context {
    fn set<T: 'static>(&mut self, mutable_state: MutableState<T>, value: T) {
        if let Some(state_info) = self.state_registry.get_mut(&mutable_state.descriptor) {
            if let Some(data) = state_info.get_data_mut(mutable_state) {
                *data = value;

                // TODO: check the validity of exising widget
                // For instance, in subscriber list may be some unexisting widget and marking dirty
                // flags will be invalid
                state_info.subscribers().for_each(|subscriber| {
                    self.dirty_registry
                        .entry(*subscriber)
                        .and_modify(|flags| *flags |= DirtyFlags::NEEDS_REBUILD)
                        .or_insert(DirtyFlags::NEEDS_REBUILD);
                });
            }
        }
    }
}

impl<'a> GetState for ScopedContext<'a> {
    fn get<T: 'static, S: Into<State<T>>>(&self, state: S) -> Option<&T> {
        let state = state.into();

        self.inner
            .get_raw(state.descriptor)
            .and_then(|val| val.downcast_ref())
    }
}

impl<'a> SetState for ScopedContext<'a> {
    fn set<T: 'static>(&mut self, mutable_state: MutableState<T>, value: T) {
        self.inner
            .set_raw(mutable_state.descriptor, Box::new(value));
    }
}

pub(crate) trait StateSubscription<Id>
where
    Id: Into<WidgetId>,
{
    /// A provided id subscribes to state changes by its descriptor.
    fn subscribe<S, T>(&mut self, id: Id, state: S)
    where
        S: Into<State<T>>,
        T: 'static;
}

impl<Id> StateSubscription<Id> for Context
where
    Id: Into<WidgetId>,
{
    fn subscribe<S, T>(&mut self, id: Id, state: S)
    where
        S: Into<State<T>>,
        T: 'static,
    {
        let state = state.into();

        if let Some(state_info) = self.state_registry.get_mut(&state.descriptor) {
            state_info.add_subscriber(id);
        }
    }
}

pub(crate) trait StyleSubscription<C, Id>
where
    C: Into<WidgetClass>,
    Id: Into<WidgetId>,
{
    /// A provided id subscribes to style changes by its class.
    fn subscribe(&mut self, id: Id, class: C);
}

impl<C, Id> StyleSubscription<C, Id> for Context
where
    Id: Into<WidgetId>,
    C: Into<WidgetClass>,
{
    fn subscribe(&mut self, id: Id, class: C) {
        let id = id.into();
        self.style_registry
            .entry(class.into())
            .and_modify(|style_info| style_info.add_subscriber(id))
            .or_insert_with(|| {
                let mut style = StyleInfo::new(WidgetStyle::Unknown);
                style.add_subscriber(id);
                style
            });
    }
}

pub trait SetStyleClass {
    /// Assigns a widget style to a style class.
    fn set_style_class<Class>(&mut self, class: Class, style: WidgetStyle)
    where
        Class: Into<WidgetClass>;
}

impl SetStyleClass for Context {
    fn set_style_class<Class>(&mut self, class: Class, style: WidgetStyle)
    where
        Class: Into<WidgetClass>,
    {
        self.style_registry
            .entry(class.into())
            .and_modify(|style_info| {
                style_info.set_style(style.clone());

                // TODO: check the validity of exising widget
                // For instance, in subscriber list may be some unexisting widget and marking dirty
                // flags will be invalid
                style_info.subscribers().for_each(|subscriber| {
                    self.dirty_registry
                        .entry(*subscriber)
                        .and_modify(|flags| *flags |= DirtyFlags::NEEDS_UPDATE_STYLES)
                        .or_insert(DirtyFlags::NEEDS_UPDATE_STYLES);
                });
            })
            .or_insert_with(|| StyleInfo::new(style));
    }
}

pub trait Tick {
    fn tick(&mut self, delta_ns: u128);
}

impl Tick for Context {
    fn tick(&mut self, delta_ns: u128) {
        for (widget_id, animation_progress) in &mut self.animation_regirsty {
            animation_progress.tick(delta_ns);

            if !animation_progress.flag_on_change.is_empty() {
                self.dirty_registry
                    .entry(*widget_id)
                    .and_modify(|flags| *flags |= animation_progress.flag_on_change)
                    .or_insert(animation_progress.flag_on_change);
            }
        }
    }
}

pub(crate) trait GenerateId {
    fn generate_id(&mut self) -> WidgetId;
}

impl GenerateId for Context {
    fn generate_id(&mut self) -> WidgetId {
        let id = self.id_counter;
        self.id_counter += 1;
        id.into()
    }
}

pub(crate) trait RegisterKey<K, Id>
where
    K: Into<WidgetKey>,
    Id: Into<WidgetId>,
{
    fn register_key(&mut self, key: K, id: Id);
}

impl<K, Id> RegisterKey<K, Id> for Context
where
    K: Into<WidgetKey>,
    Id: Into<WidgetId>,
{
    fn register_key(&mut self, key: K, id: Id) {
        self.key_registry.insert(key.into(), id.into());
    }
}

pub(crate) trait ManageDirtyFlags<Id>
where
    Id: Into<WidgetId>,
{
    fn get_dirty_flags(&self, id: Id) -> DirtyFlags;
    fn set_dirty_flags(&mut self, id: Id, dirty_flags: DirtyFlags);
    fn remove_dirty_flags(&mut self, id: Id);
}

impl<Id> ManageDirtyFlags<Id> for Context
where
    Id: Into<WidgetId>,
{
    fn get_dirty_flags(&self, id: Id) -> DirtyFlags {
        self.dirty_registry
            .get(&id.into())
            .cloned()
            .unwrap_or_else(DirtyFlags::empty)
    }

    fn set_dirty_flags(&mut self, id: Id, dirty_flags: DirtyFlags) {
        if dirty_flags.is_empty() {
            self.remove_dirty_flags(id);
        } else {
            self.dirty_registry.insert(id.into(), dirty_flags);
        }
    }

    fn remove_dirty_flags(&mut self, id: Id) {
        self.dirty_registry.remove(&id.into());
    }
}

pub(crate) trait GetStyle {
    fn get_style(&self, class: &WidgetClass) -> Option<&WidgetStyle>;
}

impl GetStyle for Context {
    fn get_style(&self, class: &WidgetClass) -> Option<&WidgetStyle> {
        self.style_registry.get(class).map(StyleInfo::get_style)
    }
}

pub(crate) trait GetFont {
    fn get_font(&self) -> skia_safe::textlayout::FontCollection;
}

impl GetFont for Context {
    fn get_font(&self) -> skia_safe::textlayout::FontCollection {
        self.font_collection.clone()
    }
}

pub(crate) trait SaveIntrinsic<T, Id>
where
    T: Default + Copy,
    Id: Into<WidgetId>,
{
    fn save(&mut self, id: Id, intrinsic: Intrinsic<T>);
}

pub(crate) trait LoadIntrinsic<T, Id>
where
    T: Default + Copy,
    Id: Into<WidgetId>,
{
    fn load(&self, id: Id) -> Option<Intrinsic<T>>;
}

pub(crate) trait ManageIntrinsic<T, Id>:
    SaveIntrinsic<T, Id> + LoadIntrinsic<T, Id>
where
    T: Default + Copy,
    Id: Into<WidgetId>,
{
}

impl<C, T, Id> ManageIntrinsic<T, Id> for C
where
    T: Default + Copy,
    Id: Into<WidgetId>,
    C: SaveIntrinsic<T, Id> + LoadIntrinsic<T, Id>,
{
}

pub(crate) trait SaveExtent<T, Id>
where
    T: Default + Copy,
    Id: Into<WidgetId>,
{
    fn save(&mut self, id: Id, extent: Extent<T>);
}

pub(crate) trait LoadExtent<T, Id>
where
    T: Default + Copy,
    Id: Into<WidgetId>,
{
    fn load(&self, id: Id) -> Option<Extent<T>>;
}

pub(crate) trait ManageExtent<T, Id>: SaveExtent<T, Id> + LoadExtent<T, Id>
where
    T: Default + Copy,
    Id: Into<WidgetId>,
{
}

impl<C, T, Id> ManageExtent<T, Id> for C
where
    T: Default + Copy,
    Id: Into<WidgetId>,
    C: SaveExtent<T, Id> + LoadExtent<T, Id>,
{
}

pub(crate) trait SaveConstraints<T, Id>
where
    T: Default + Copy,
    Id: Into<WidgetId>,
{
    fn save(&mut self, id: Id, constraints: Constraints<Extent<T>>);
}

pub(crate) trait LoadConstraints<T, Id>
where
    T: Default + Copy,
    Id: Into<WidgetId>,
{
    fn load(&self, id: Id) -> Option<Constraints<Extent<T>>>;
}

pub(crate) trait ManageConstraints<T, Id>:
    SaveConstraints<T, Id> + LoadConstraints<T, Id>
where
    T: Default + Copy,
    Id: Into<WidgetId>,
{
}

impl<C, T, Id> ManageConstraints<T, Id> for C
where
    T: Default + Copy,
    Id: Into<WidgetId>,
    C: SaveConstraints<T, Id> + LoadConstraints<T, Id>,
{
}

impl<Id> SaveIntrinsic<f32, Id> for Context
where
    Id: Into<WidgetId>,
{
    fn save(&mut self, id: Id, intrinsic: Intrinsic<f32>) {
        self.measure_cache
            .entry(id.into())
            .and_modify(|cache| {
                cache.intrinsic.replace(intrinsic);
            })
            .or_insert(MeasureCache {
                intrinsic: intrinsic.into(),
                ..Default::default()
            });
    }
}

impl<Id> LoadIntrinsic<f32, Id> for Context
where
    Id: Into<WidgetId>,
{
    fn load(&self, id: Id) -> Option<Intrinsic<f32>> {
        self.measure_cache
            .get(&id.into())
            .and_then(|cache| cache.intrinsic)
    }
}

impl<Id> SaveExtent<f32, Id> for Context
where
    Id: Into<WidgetId>,
{
    fn save(&mut self, id: Id, extent: Extent<f32>) {
        self.measure_cache
            .entry(id.into())
            .and_modify(|cache| {
                cache.extent.replace(extent);
            })
            .or_insert(MeasureCache {
                extent: extent.into(),
                ..Default::default()
            });
    }
}

impl<Id> LoadExtent<f32, Id> for Context
where
    Id: Into<WidgetId>,
{
    fn load(&self, id: Id) -> Option<Extent<f32>> {
        self.measure_cache
            .get(&id.into())
            .and_then(|cache| cache.extent)
    }
}

impl<Id> SaveConstraints<f32, Id> for Context
where
    Id: Into<WidgetId>,
{
    fn save(&mut self, id: Id, constraints: Constraints<Extent<f32>>) {
        self.measure_cache
            .entry(id.into())
            .and_modify(|cache| {
                cache.constraints.replace(constraints);
            })
            .or_insert(MeasureCache {
                constraints: constraints.into(),
                ..Default::default()
            });
    }
}

impl<Id> LoadConstraints<f32, Id> for Context
where
    Id: Into<WidgetId>,
{
    fn load(&self, id: Id) -> Option<Constraints<Extent<f32>>> {
        self.measure_cache
            .get(&id.into())
            .and_then(|cache| cache.constraints)
    }
}

pub(crate) trait ManageAnimationRegistry<Id>
where
    Id: Into<WidgetId>,
{
    fn register_animation(&mut self, id: Id, animation_progress: AnimationProgress);
    fn remove_animation(&mut self, id: Id);
    fn reverse_animation(&mut self, id: Id);
}

pub(crate) trait AnimationQuery<Id>
where
    Id: Into<WidgetId>,
{
    fn animation_progress(&self, id: Id) -> Option<f32>;
    fn is_finished(&self, id: Id) -> bool;
}

impl<Id> ManageAnimationRegistry<Id> for Context
where
    Id: Into<WidgetId>,
{
    fn register_animation(&mut self, id: Id, animation_progress: AnimationProgress) {
        self.animation_regirsty
            .insert(id.into(), animation_progress);
    }

    fn remove_animation(&mut self, id: Id) {
        self.animation_regirsty.remove(&id.into());
    }

    fn reverse_animation(&mut self, id: Id) {
        self.animation_regirsty
            .entry(id.into())
            .and_modify(AnimationProgress::reverse);
    }
}

impl<Id> AnimationQuery<Id> for Context
where
    Id: Into<WidgetId>,
{
    fn animation_progress(&self, id: Id) -> Option<f32> {
        self.animation_regirsty
            .get(&id.into())
            .map(AnimationProgress::progress)
    }

    fn is_finished(&self, id: Id) -> bool {
        self.animation_regirsty
            .get(&id.into())
            .map(AnimationProgress::is_finished)
            .unwrap_or(true)
    }
}
