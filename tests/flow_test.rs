use modularitea_libs::domain::TaskPhase;
use modularitea_libs::loader::TomlLoader;
use modularitea_libs::planner::TaskPlanner;

#[test]
fn test_full_execution_flow() {
    let toml_input = r#"
        [meta]
        name = "gaming-station"
        description = "Setup for high-performance gaming"
        version = "1.0.0"
        author = "TeaLinux Team"

        [packages]
        install = ["steam", "lutris", "gamemode"]
        aur = ["proton-ge-custom-bin"]
        remove = ["xgames-useless-demo"]

        [services]
        enable = ["bluetooth"]
        disable = ["cups"] # No printing while gaming!

        [grub]
        timeout = 5
        cmdline_linux = ["preempt=full"]
        theme = "/usr/share/grub/themes/tealinux-gaming"

        [filesystem]
        mkdir = ["/home/user/Games"]
    "#;

    // 2. Load the profile (Loader Layer)
    let profile = TomlLoader::load_from_string(toml_input).expect("Failed to parse valid TOML");

    assert_eq!(profile.meta.name, "gaming-station");
    assert!(
        profile.requires_root(),
        "Profile with packages should require root"
    );

    // 3. Create execution plan (Planner Layer)
    let plan = TaskPlanner::plan(&profile).expect("Failed to generate plan");

    // 4. Validate the Plan
    assert_eq!(plan.profile_name, "gaming-station");
    assert!(plan.tasks.len() > 0);

    // 5. Verify Task Phases and Ordering
    // Order should be: Packages -> Services -> Filesystem -> Configure (Grub)
    // We'll iterate and check if phases generally increase
    let mut current_phase = TaskPhase::Prepare;
    for task in &plan.tasks {
        assert!(
            task.phase >= current_phase,
            "Task phase execution order violation: {:?} came after {:?}",
            task.phase,
            current_phase
        );
        current_phase = task.phase;
    }

    // 6. specific validation of tasks
    let pkg_tasks: Vec<_> = plan
        .tasks
        .iter()
        .filter(|t| t.phase == TaskPhase::Packages)
        .collect();

    let svc_tasks: Vec<_> = plan
        .tasks
        .iter()
        .filter(|t| t.phase == TaskPhase::Services)
        .collect();

    let grub_tasks: Vec<_> = plan
        .tasks
        .iter()
        .filter(|t| t.name.contains("GRUB"))
        .collect();

    // Check Package Tasks
    // Expect: Remove, Install(Official), Install(AUR)
    assert!(
        pkg_tasks.iter().any(|t| t.name.contains("Remove")),
        "Should have remove task"
    );
    assert!(
        pkg_tasks
            .iter()
            .any(|t| t.name.contains("Install official")),
        "Should have install official task"
    );
    assert!(
        pkg_tasks.iter().any(|t| t.name.contains("Install AUR")),
        "Should have AUR task"
    );

    // Check Service Tasks
    assert!(
        svc_tasks.iter().any(|t| t.name.contains("Enable")),
        "Should have enable service task"
    );
    assert!(
        svc_tasks.iter().any(|t| t.name.contains("Disable")),
        "Should have disable service task"
    );

    // Check GRUB Tasks
    assert!(grub_tasks.len() >= 3); // Theme, Timeout, Cmdline, maybe Regen
}
