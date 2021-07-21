use super::{Schedule, StageLabel, SystemDescriptor};
use crate::system::Command;
use crate::world::World;

#[derive(Default)]
pub struct SchedulerCommandQueue {
    items: Vec<Box<dyn SchedulerCommand>>,
}

impl SchedulerCommandQueue {
    pub fn push<C>(&mut self, command: C)
    where
        C: SchedulerCommand,
    {
        self.items.push(Box::new(command));
    }

    pub fn apply(&mut self, schedule: &mut Schedule) {
        for command in self.items.drain(..) {
            command.write(schedule);
        }
    }

    pub fn transfer(&mut self, queue: &mut SchedulerCommandQueue) {
        queue.items.extend(self.items.drain(..));
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// A [`Schedule`] mutation.
pub trait SchedulerCommand: Send + Sync + 'static {
    fn write(self: Box<Self>, schedule: &mut Schedule);
}

impl<T> Command for T
where
    T: SchedulerCommand,
{
    fn write(self, world: &mut World) {
        world.scheduler_commands.push(self);
    }
}

pub struct InsertSystem<S>
where
    S: StageLabel,
{
    pub system: SystemDescriptor,
    pub stage_label: S,
}

impl<S> SchedulerCommand for InsertSystem<S>
where
    S: StageLabel,
{
    fn write(self: Box<Self>, schedule: &mut Schedule) {
        schedule.add_system_to_stage(self.stage_label, self.system);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        schedule::{
            InsertSystem, IntoSystemDescriptor, Schedule, SchedulerCommandQueue, SystemStage,
        },
        system::Commands,
        world::World,
    };

    #[test]
    fn insert_system() {
        fn sample_system(mut _commands: Commands) {}
        let mut schedule = Schedule::default();
        schedule.add_stage("test", SystemStage::parallel());
        let mut queue = SchedulerCommandQueue::default();
        queue.push(InsertSystem {
            system: sample_system.into_descriptor(),
            stage_label: "test",
        });
        queue.apply(&mut schedule);

        let stage = schedule.get_stage::<SystemStage>(&"test").unwrap();
        assert_eq!(stage.parallel_systems().len(), 1);
    }

    #[test]
    fn insert_system_from_system() {
        fn sample_system(mut commands: Commands) {
            commands.insert_system(|| {}, "test");
        }

        let mut world = World::default();
        let mut schedule = Schedule::default();
        schedule.add_stage("test", SystemStage::parallel());
        schedule.add_system_to_stage("test", sample_system);
        schedule.run_once(&mut world);

        let stage = schedule.get_stage::<SystemStage>(&"test").unwrap();
        assert_eq!(stage.parallel_systems().len(), 2);
    }
}
