use enum_map::{EnumMap, enum_map};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

pub struct Benchmark {
    items: EnumMap<BenchmarkItem, VecDeque<Duration>>,
    capacity: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, enum_map::Enum)]
pub enum BenchmarkItem {
    CreateDom,
    CreateWidget,
    UpdateWidget,
    LayoutMeasure,
    LayoutArrange,
    WidgetRender,
    GpuDrivenRender,
}

impl BenchmarkItem {
    pub fn all() -> &'static [BenchmarkItem] {
        &[
            BenchmarkItem::CreateDom,
            BenchmarkItem::UpdateWidget,
            BenchmarkItem::LayoutMeasure,
            BenchmarkItem::LayoutArrange,
            BenchmarkItem::WidgetRender,
            BenchmarkItem::GpuDrivenRender,
        ]
    }

    pub fn print_names(&self) -> &str {
        match self {
            BenchmarkItem::CreateDom => "Create Dom",
            BenchmarkItem::CreateWidget => "Create Widget",
            BenchmarkItem::UpdateWidget => "Update Widget",
            BenchmarkItem::LayoutMeasure => "Layout Measure",
            BenchmarkItem::LayoutArrange => "Layout Arrange",
            BenchmarkItem::WidgetRender => "Widget Render",
            BenchmarkItem::GpuDrivenRender => "GPU Driven Render",
        }
    }
}

impl Benchmark {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: enum_map! {
                BenchmarkItem::CreateDom => VecDeque::with_capacity(capacity),
                BenchmarkItem::CreateWidget => VecDeque::with_capacity(capacity),
                BenchmarkItem::UpdateWidget => VecDeque::with_capacity(capacity),
                BenchmarkItem::LayoutMeasure => VecDeque::with_capacity(capacity),
                BenchmarkItem::LayoutArrange => VecDeque::with_capacity(capacity),
                BenchmarkItem::WidgetRender => VecDeque::with_capacity(capacity),
                BenchmarkItem::GpuDrivenRender => VecDeque::with_capacity(capacity),
            },
            capacity,
        }
    }

    pub fn clear(&mut self) {
        for buffer in self.items.values_mut() {
            buffer.clear();
        }
    }
}

impl Benchmark {
    #[inline]
    pub fn with<R>(&mut self, item: BenchmarkItem, f: impl FnOnce() -> R) -> R {
        let start = Instant::now();
        let r = f();
        let duration = start.elapsed();
        let buffer = &mut self.items[item];
        if buffer.len() == self.capacity {
            buffer.pop_front();
        }
        buffer.push_back(duration);

        r
    }

    pub async fn with_async<R, F>(&mut self, item: BenchmarkItem, f: F) -> R
    where
        F: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let r = f.await;
        let duration = start.elapsed();
        let buffer = &mut self.items[item];
        if buffer.len() == self.capacity {
            buffer.pop_front();
        }
        buffer.push_back(duration);

        r
    }

    pub async fn with_create_dom<R>(&mut self, f: impl std::future::Future<Output = R>) -> R {
        self.with_async(BenchmarkItem::CreateDom, f).await
    }

    pub fn with_create_widget<R>(&mut self, f: impl FnOnce() -> R) -> R {
        self.with(BenchmarkItem::CreateWidget, f)
    }

    pub async fn with_update_widget<R>(&mut self, f: impl std::future::Future<Output = R>) -> R {
        self.with_async(BenchmarkItem::UpdateWidget, f).await
    }

    pub fn with_layout_measure<R>(&mut self, f: impl FnOnce() -> R) -> R {
        self.with(BenchmarkItem::LayoutMeasure, f)
    }

    pub fn with_layout_arrange<R>(&mut self, f: impl FnOnce() -> R) -> R {
        self.with(BenchmarkItem::LayoutArrange, f)
    }

    pub fn with_widget_render<R>(&mut self, f: impl FnOnce() -> R) -> R {
        self.with(BenchmarkItem::WidgetRender, f)
    }

    pub fn with_gpu_driven_render<R>(&mut self, f: impl FnOnce() -> R) -> R {
        self.with(BenchmarkItem::GpuDrivenRender, f)
    }
}

impl Benchmark {
    pub fn last_time(&self, item: BenchmarkItem) -> Option<Time> {
        self.items[item]
            .back()
            .map(|d| Time::from_micros(d.as_micros()))
    }

    pub fn last_time_create_dom(&self) -> Option<Time> {
        self.last_time(BenchmarkItem::CreateDom)
    }

    pub fn last_time_create_widget(&self) -> Option<Time> {
        self.last_time(BenchmarkItem::CreateWidget)
    }

    pub fn last_time_update_widget(&self) -> Option<Time> {
        self.last_time(BenchmarkItem::UpdateWidget)
    }

    pub fn last_time_layout_measure(&self) -> Option<Time> {
        self.last_time(BenchmarkItem::LayoutMeasure)
    }

    pub fn last_time_layout_arrange(&self) -> Option<Time> {
        self.last_time(BenchmarkItem::LayoutArrange)
    }

    pub fn last_time_widget_render(&self) -> Option<Time> {
        self.last_time(BenchmarkItem::WidgetRender)
    }

    pub fn last_time_gpu_driven_render(&self) -> Option<Time> {
        self.last_time(BenchmarkItem::GpuDrivenRender)
    }

    pub fn average_time(&self, item: BenchmarkItem) -> Option<Time> {
        let buffer = &self.items[item];
        if buffer.is_empty() {
            return None;
        }
        let total_micros: u128 = buffer.iter().map(|d| d.as_micros()).sum();
        let avg_micros = total_micros / (buffer.len() as u128);
        Some(Time::from_micros(avg_micros))
    }

    pub fn average_time_create_dom(&self) -> Option<Time> {
        self.average_time(BenchmarkItem::CreateDom)
    }

    pub fn average_time_create_widget(&self) -> Option<Time> {
        self.average_time(BenchmarkItem::CreateWidget)
    }

    pub fn average_time_update_widget(&self) -> Option<Time> {
        self.average_time(BenchmarkItem::UpdateWidget)
    }

    pub fn average_time_layout_measure(&self) -> Option<Time> {
        self.average_time(BenchmarkItem::LayoutMeasure)
    }

    pub fn average_time_layout_arrange(&self) -> Option<Time> {
        self.average_time(BenchmarkItem::LayoutArrange)
    }

    pub fn average_time_widget_render(&self) -> Option<Time> {
        self.average_time(BenchmarkItem::WidgetRender)
    }

    pub fn average_time_gpu_driven_render(&self) -> Option<Time> {
        self.average_time(BenchmarkItem::GpuDrivenRender)
    }
}

impl Benchmark {
    pub fn print(&self) {
        print!("Benchmarks: ");
        for &item in BenchmarkItem::all() {
            print!(
                "| {}: last {}, avr {} ",
                item.print_names(),
                self.last_time(item)
                    .map(|t| t.to_string())
                    .unwrap_or("-".to_string()),
                self.average_time(item)
                    .map(|t| t.to_string())
                    .unwrap_or("-".to_string())
            );
        }
    }
}

pub enum Time {
    Second(u32),
    Millisecond(u32),
    Microsecond(u32),
}

impl Time {
    pub fn from_micros(micros: u128) -> Self {
        if micros <= 10_000 {
            Time::Microsecond(micros as u32)
        } else if micros <= 10_000_000 {
            Time::Millisecond((micros / 1_000) as u32)
        } else {
            Time::Second((micros / 1_000_000) as u32)
        }
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Time::Second(time) => write!(f, "{time:>4}s "),
            Time::Millisecond(time) => write!(f, "{time:>4}ms"),
            Time::Microsecond(time) => write!(f, "{time:>4}µs"),
        }
    }
}
