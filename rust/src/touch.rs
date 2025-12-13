use crate::event_bus::MobileEvent;
use notan_core::events::Event;

/// Convert mobile touch events to notan events
pub fn process_events(mobile_event: &MobileEvent, _scale: f64) -> Option<Event> {
    match mobile_event {
        MobileEvent::Touch { x, y, action } => {
            // Convert touch coordinates (already in logical pixels) to screen coordinates
            let screen_x = *x as f32;
            let screen_y = *y as f32;

            match action {
                0 => {
                    // Touch down - convert to mouse button press
                    // Note: MouseButton might not be accessible, using TouchStart instead
                    Some(Event::TouchStart {
                        id: 0,
                        x: screen_x,
                        y: screen_y,
                    })
                }
                1 => {
                    // Touch up - convert to mouse button release
                    Some(Event::TouchEnd {
                        id: 0,
                        x: screen_x,
                        y: screen_y,
                    })
                }
                2 => {
                    // Touch move - convert to mouse motion
                    Some(Event::TouchMove {
                        id: 0,
                        x: screen_x,
                        y: screen_y,
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

