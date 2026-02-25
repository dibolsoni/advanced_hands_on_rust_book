#[macro_export]
macro_rules! add_phase {
    (
        $app:expr, $type:ty, $phase:expr,
        start => [ $( $start:expr ),* ],
        run => [ $( $run:expr ),* ],
        exit => [ $( $exit:expr ),* ]
    ) => {
        $($app.add_systems(
            bevy::prelude::OnEnter::<$type>($phase),
            $start
        );)*
        $($app.add_systems(
            bevy::prelude::Update, $run.run_if(in_state($phase))
        );)*
        $($app.add_systems(
            bevy::prelude::OnExit::<$type>($phase),
            $exit
        );)*
    };
}
