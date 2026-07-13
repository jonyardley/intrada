# Graph Report - .  (2026-07-13)

## Corpus Check
- Large corpus: 710 files · ~873,283 words. Semantic extraction will be expensive (many Claude tokens). Consider running on a subfolder.

## Summary
- 6040 nodes · 12714 edges · 1 communities
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 665 edges (avg confidence: 0.71)
- Token cost: 0 input · 2,626,283 output

## Community Hubs (Navigation)
- Whole repo (no clustering)

## God Nodes (most connected - your core abstractions)
1. `e5` - 166 edges
2. `ApiError` - 158 edges
3. `update()` - 127 edges
4. `AppState` - 79 edges
5. `u()` - 75 edges
6. `i()` - 74 edges
7. `n()` - 68 edges
8. `s()` - 67 edges
9. `SwiftUI` - 66 edges
10. `Model` - 63 edges

## Surprising Connections (you probably didn't know these)
- `Principle II: Testing Standards` --semantically_similar_to--> `Offline-First Invariants`  [INFERRED] [semantically similar]
  .specify/memory/constitution.md → CLAUDE.md
- `Principle V: Architecture Integrity` --semantically_similar_to--> `Crux Capabilities Pattern`  [INFERRED] [semantically similar]
  .specify/memory/constitution.md → CLAUDE.md
- `SaveLibrarySort AppEffect` --semantically_similar_to--> `StorageEffect`  [INFERRED] [semantically similar]
  docs/superpowers/plans/2026-06-01-library-sort.md → specs/_archive/001-music-library/contracts/library-api.md
- `Principle IV: Performance Requirements` --conceptually_related_to--> `intrada Development Guidelines (CLAUDE.md)`  [INFERRED]
  .specify/memory/constitution.md → CLAUDE.md
- `Light-mode 'paper' exploration — design notes` --conceptually_related_to--> `Native SwiftUI iOS pivot (Crux core, local-first, retiring Tauri)`  [INFERRED]
  design/light-mode-exploration.md → docs/roadmap.md

## Import Cycles
- 2-file cycle: `crates/intrada-api/src/rate_limit.rs -> crates/intrada-api/src/state.rs -> crates/intrada-api/src/rate_limit.rs`
- 2-file cycle: `crates/intrada-api/src/auth.rs -> crates/intrada-api/src/state.rs -> crates/intrada-api/src/auth.rs`
- 2-file cycle: `crates/intrada-core/src/app.rs -> crates/intrada-core/src/http.rs -> crates/intrada-core/src/app.rs`
- 2-file cycle: `crates/intrada-core/src/domain/session.rs -> crates/intrada-core/src/model.rs -> crates/intrada-core/src/domain/session.rs`
- 2-file cycle: `crates/intrada-core/src/domain/mcp_audit.rs -> crates/intrada-core/src/model.rs -> crates/intrada-core/src/domain/mcp_audit.rs`
- 3-file cycle: `crates/intrada-api/src/auth.rs -> crates/intrada-api/src/state.rs -> crates/intrada-api/src/rate_limit.rs -> crates/intrada-api/src/auth.rs`

## Hyperedges (group relationships)
- **iOS-Only Pivot Governance** — claude_dev_guidelines_ios_only_focus, github_dependabot_ios_pivot_ignore, github_workflows_ci_ios_pivot_disabled_jobs [INFERRED 0.85]
- **Offline-First Data Integrity Guarantees** — claude_dev_guidelines_offline_first_invariants, claude_dev_guidelines_local_data_migrations, claude_dev_guidelines_json_bincode_ffi_bug [EXTRACTED 0.90]
- **Native iOS TestFlight Release Pipeline** — github_workflows_release_testflight, github_workflows_ci_native_ios_job, setup_testflight_bootstrap [EXTRACTED 0.90]
- **Design file sync & single-ownership process** — design_design_process_md, design_claude_md, design_handover_md, concept_single_ownership_per_surface, concept_fold_in_ratchet [INFERRED 0.85]
- **dc-import shared-screen pattern across design files** — design_intrada_design_system_dc_html, design_linked_exercises_dc_html, design_focus_player_dc_html, design_session_summary_dc_html, concept_dc_import_pattern [EXTRACTED 0.95]
- **Self-Determination Theory evidence supporting autonomy-first design** — docs_research_foundation_md, ryan_deci_2000, valenzuela_codina_pestana_2018, bonneville_roussy_evans_2024, concept_spend_friction_deliberately [INFERRED 0.75]
- **001-music-library SpecKit Document Bundle** — specs__archive_001_music_library_checklists_requirements, specs__archive_001_music_library_contracts_library_api, specs__archive_001_music_library_data_model, specs__archive_001_music_library_plan, specs__archive_001_music_library_quickstart, specs__archive_001_music_library_research, specs__archive_001_music_library_spec, specs__archive_001_music_library_tasks [INFERRED 0.90]
- **002-ci-cd SpecKit Document Bundle** — specs__archive_002_ci_cd_checklists_requirements, specs__archive_002_ci_cd_plan, specs__archive_002_ci_cd_quickstart, specs__archive_002_ci_cd_research, specs__archive_002_ci_cd_spec, specs__archive_002_ci_cd_tasks [INFERRED 0.90]
- **003-leptos-app-mvp SpecKit Document Bundle (partial, this chunk)** — specs__archive_003_leptos_app_mvp_checklists_requirements, specs__archive_003_leptos_app_mvp_data_model, specs__archive_003_leptos_app_mvp_plan, specs__archive_003_leptos_app_mvp_quickstart, specs__archive_003_leptos_app_mvp_research [INFERRED 0.90]
- **Feature 004 documentation set (spec/plan/tasks/data-model/events/research/quickstart/checklist)** — specs_archive_004_library_detail_editing_spec, specs_archive_004_library_detail_editing_plan, specs_archive_004_library_detail_editing_tasks, specs_archive_004_library_detail_editing_data_model, specs_archive_004_library_detail_editing_contracts_events, specs_archive_004_library_detail_editing_research, specs_archive_004_library_detail_editing_quickstart, specs_archive_004_library_detail_editing_checklists_requirements [EXTRACTED 1.00]
- **Feature 005 documentation set (spec/plan/tasks/research/quickstart/checklist)** — specs_archive_005_component_architecture_spec, specs_archive_005_component_architecture_plan, specs_archive_005_component_architecture_tasks, specs_archive_005_component_architecture_research, specs_archive_005_component_architecture_quickstart, specs_archive_005_component_architecture_checklists_requirements [EXTRACTED 1.00]
- **Feature 006 documentation set (spec/plan/tasks/research/quickstart/checklist)** — specs_archive_006_ui_primitives_spec, specs_archive_006_ui_primitives_plan, specs_archive_006_ui_primitives_tasks, specs_archive_006_ui_primitives_research, specs_archive_006_ui_primitives_quickstart, specs_archive_006_ui_primitives_checklists_requirements [EXTRACTED 1.00]
- **007 Crux & Leptos Upgrade — archived spec doc set** — specs_archive_007_crux_leptos_upgrade_spec, specs_archive_007_crux_leptos_upgrade_plan, specs_archive_007_crux_leptos_upgrade_research, specs_archive_007_crux_leptos_upgrade_quickstart, specs_archive_007_crux_leptos_upgrade_tasks, specs_archive_007_crux_leptos_upgrade_checklists_requirements [EXTRACTED 1.00]
- **008 URL Routing — archived spec doc set** — specs_archive_008_url_routing_spec, specs_archive_008_url_routing_plan, specs_archive_008_url_routing_research, specs_archive_008_url_routing_quickstart, specs_archive_008_url_routing_tasks, specs_archive_008_url_routing_checklists_requirements [EXTRACTED 1.00]
- **009 Unified Library Item Form — archived spec doc set** — specs_archive_009_unified_library_form_spec, specs_archive_009_unified_library_form_plan, specs_archive_009_unified_library_form_research, specs_archive_009_unified_library_form_quickstart, specs_archive_009_unified_library_form_tasks, specs_archive_009_unified_library_form_checklists_requirements, specs_archive_009_unified_library_form_data_model [EXTRACTED 1.00]
- **011 JSON File Persistence — Spec-Kit Document Set** — specs_archive_011_json_persistence_research, specs_archive_011_json_persistence_spec, specs_archive_011_json_persistence_tasks [INFERRED 0.85]
- **012 Practice Sessions — Spec-Kit Document Set** — specs_archive_012_practice_sessions_checklists_requirements, specs_archive_012_practice_sessions_data_model, specs_archive_012_practice_sessions_plan, specs_archive_012_practice_sessions_quickstart, specs_archive_012_practice_sessions_research, specs_archive_012_practice_sessions_spec, specs_archive_012_practice_sessions_tasks [INFERRED 0.85]
- **013 Web UI Testing & E2E Infrastructure — Spec-Kit Document Set** — specs_archive_013_web_testing_checklists_requirements, specs_archive_013_web_testing_data_model, specs_archive_013_web_testing_plan, specs_archive_013_web_testing_quickstart, specs_archive_013_web_testing_research, specs_archive_013_web_testing_spec, specs_archive_013_web_testing_tasks [INFERRED 0.85]
- **015 Rework Sessions Spec Bundle** — specs_archive_015_rework_sessions_spec, specs_archive_015_rework_sessions_plan, specs_archive_015_rework_sessions_data_model, specs_archive_015_rework_sessions_tasks, specs_archive_015_rework_sessions_research, specs_archive_015_rework_sessions_contracts_session_events, specs_archive_015_rework_sessions_quickstart, specs_archive_015_rework_sessions_checklists_requirements [EXTRACTED 1.00]
- **016 Glassmorphism Responsive Spec Bundle** — specs_archive_016_glassmorphism_responsive_spec, specs_archive_016_glassmorphism_responsive_plan, specs_archive_016_glassmorphism_responsive_quickstart, specs_archive_016_glassmorphism_responsive_research, specs_archive_016_glassmorphism_responsive_tasks, specs_archive_016_glassmorphism_responsive_checklists_requirements [EXTRACTED 1.00]
- **019 Improve CI/CD Spec Bundle** — specs_archive_019_improve_cicd_spec, specs_archive_019_improve_cicd_plan, specs_archive_019_improve_cicd_quickstart, specs_archive_019_improve_cicd_research, specs_archive_019_improve_cicd_tasks, specs_archive_019_improve_cicd_checklists_requirements [EXTRACTED 1.00]
- **Archived Feature Spec 020: API Server document set** — specs_archive_020_api_server_plan, specs_archive_020_api_server_spec, specs_archive_020_api_server_research, specs_archive_020_api_server_data_model, specs_archive_020_api_server_quickstart, specs_archive_020_api_server_tasks, specs_archive_020_api_server_contracts_pieces, specs_archive_020_api_server_contracts_exercises, specs_archive_020_api_server_contracts_sessions, specs_archive_020_api_server_contracts_health [EXTRACTED 1.00]
- **Archived Feature Spec 021: API Sync document set** — specs_archive_021_api_sync_plan, specs_archive_021_api_sync_spec, specs_archive_021_api_sync_research, specs_archive_021_api_sync_data_model, specs_archive_021_api_sync_quickstart, specs_archive_021_api_sync_tasks, specs_archive_021_api_sync_contracts_api_client, specs_archive_021_api_sync_checklists_requirements [EXTRACTED 1.00]
- **Archived Feature Spec 022: Session Item Scoring document set (partial — chunk 8)** — specs_archive_022_session_scoring_plan, specs_archive_022_session_scoring_data_model, specs_archive_022_session_scoring_contracts_api_changes, specs_archive_022_session_scoring_checklists_requirements [EXTRACTED 1.00]
- **022-session-scoring feature spec bundle** — specs_archive_022_session_scoring_spec_doc, specs_archive_022_session_scoring_research_doc, specs_archive_022_session_scoring_quickstart_doc, specs_archive_022_session_scoring_tasks_doc [EXTRACTED 1.00]
- **023-analytics-dashboard feature spec bundle** — specs_archive_023_analytics_dashboard_spec_doc, specs_archive_023_analytics_dashboard_plan_doc, specs_archive_023_analytics_dashboard_research_doc, specs_archive_023_analytics_dashboard_data_model_doc, specs_archive_023_analytics_dashboard_quickstart_doc, specs_archive_023_analytics_dashboard_contracts_api_changes_doc, specs_archive_023_analytics_dashboard_checklists_requirements_doc, specs_archive_023_analytics_dashboard_tasks_doc [EXTRACTED 1.00]
- **024-form-autocomplete feature spec bundle** — specs_archive_024_form_autocomplete_spec_doc, specs_archive_024_form_autocomplete_plan_doc, specs_archive_024_form_autocomplete_research_doc, specs_archive_024_form_autocomplete_data_model_doc, specs_archive_024_form_autocomplete_quickstart_doc, specs_archive_024_form_autocomplete_contracts_components_doc, specs_archive_024_form_autocomplete_checklists_requirements_doc, specs_archive_024_form_autocomplete_tasks_doc [EXTRACTED 1.00]
- **025-reusable-routines archived feature spec bundle** — specs__archive_025_reusable_routines_spec, specs__archive_025_reusable_routines_plan, specs__archive_025_reusable_routines_research, specs__archive_025_reusable_routines_data_model, specs__archive_025_reusable_routines_quickstart, specs__archive_025_reusable_routines_tasks [EXTRACTED 1.00]
- **026-drag-drop-builder archived feature spec bundle** — specs__archive_026_drag_drop_builder_spec, specs__archive_026_drag_drop_builder_plan, specs__archive_026_drag_drop_builder_research, specs__archive_026_drag_drop_builder_data_model, specs__archive_026_drag_drop_builder_quickstart, specs__archive_026_drag_drop_builder_tasks, specs__archive_026_drag_drop_builder_checklists_requirements, specs__archive_026_drag_drop_builder_contracts_readme [EXTRACTED 1.00]
- **048-focus-mode archived feature spec bundle** — specs__archive_048_focus_mode_spec, specs__archive_048_focus_mode_plan, specs__archive_048_focus_mode_research, specs__archive_048_focus_mode_data_model, specs__archive_048_focus_mode_quickstart, specs__archive_048_focus_mode_tasks, specs__archive_048_focus_mode_checklists_requirements, specs__archive_048_focus_mode_contracts_api_changes [EXTRACTED 1.00]
- **095-user-auth Archived Feature Spec Document Set** — specs_archive_095_user_auth_checklists_requirements_doc, specs_archive_095_user_auth_contracts_auth_changes_doc, specs_archive_095_user_auth_data_model_doc, specs_archive_095_user_auth_plan_doc, specs_archive_095_user_auth_quickstart_doc, specs_archive_095_user_auth_research_doc, specs_archive_095_user_auth_spec_doc, specs_archive_095_user_auth_tasks_doc [EXTRACTED 1.00]
- **103-repetition-counter Archived Feature Spec Document Set** — specs_archive_103_repetition_counter_checklists_requirements_doc, specs_archive_103_repetition_counter_contracts_api_changes_doc, specs_archive_103_repetition_counter_data_model_doc, specs_archive_103_repetition_counter_plan_doc, specs_archive_103_repetition_counter_quickstart_doc, specs_archive_103_repetition_counter_research_doc, specs_archive_103_repetition_counter_spec_doc, specs_archive_103_repetition_counter_tasks_doc [EXTRACTED 1.00]
- **104-rep-history Archived Feature Spec Document Set** — specs_archive_104_rep_history_checklists_requirements_doc, specs_archive_104_rep_history_contracts_api_changes_doc, specs_archive_104_rep_history_data_model_doc, specs_archive_104_rep_history_plan_doc, specs_archive_104_rep_history_quickstart_doc, specs_archive_104_rep_history_research_doc [EXTRACTED 1.00]
- **105-tempo-tracking archived feature spec bundle** — specs_archive_105_tempo_tracking_spec, specs_archive_105_tempo_tracking_plan, specs_archive_105_tempo_tracking_research, specs_archive_105_tempo_tracking_data_model, specs_archive_105_tempo_tracking_contracts_api, specs_archive_105_tempo_tracking_quickstart, specs_archive_105_tempo_tracking_tasks, specs_archive_105_tempo_tracking_checklists_requirements [EXTRACTED 1.00]
- **151-tempo-progress-charts archived feature spec bundle** — specs_archive_151_tempo_progress_charts_spec, specs_archive_151_tempo_progress_charts_plan, specs_archive_151_tempo_progress_charts_research, specs_archive_151_tempo_progress_charts_data_model, specs_archive_151_tempo_progress_charts_contracts_readme, specs_archive_151_tempo_progress_charts_quickstart, specs_archive_151_tempo_progress_charts_tasks, specs_archive_151_tempo_progress_charts_checklists_requirements [EXTRACTED 1.00]
- **104-rep-history archived feature spec bundle** — specs_archive_104_rep_history_spec, specs_archive_104_rep_history_tasks, 104_rep_history_repaction, 104_rep_history_hide_show_preserve [INFERRED 0.85]
- **153 Weekly Practice Summary archived SpecKit bundle** — specs__archive_153_weekly_practice_summary_quickstart, specs__archive_153_weekly_practice_summary_research, specs__archive_153_weekly_practice_summary_spec, specs__archive_153_weekly_practice_summary_tasks [EXTRACTED 0.95]
- **154 Session Week Strip archived SpecKit bundle** — specs__archive_154_session_week_strip_checklists_requirements, specs__archive_154_session_week_strip_contracts_readme, specs__archive_154_session_week_strip_data_model, specs__archive_154_session_week_strip_plan, specs__archive_154_session_week_strip_quickstart, specs__archive_154_session_week_strip_research, specs__archive_154_session_week_strip_spec, specs__archive_154_session_week_strip_tasks [EXTRACTED 0.95]
- **269 Teacher Capture archived SpecKit bundle** — specs__archive_269_teacher_capture_checklists_requirements, specs__archive_269_teacher_capture_contracts_api, specs__archive_269_teacher_capture_data_model, specs__archive_269_teacher_capture_plan, specs__archive_269_teacher_capture_quickstart, specs__archive_269_teacher_capture_research, specs__archive_269_teacher_capture_spec, specs__archive_269_teacher_capture_tasks [EXTRACTED 0.95]
- **Native SwiftUI + Crux practice-player architecture (Phase A persistence -> player spine -> shell)** — specs_native_ios_doc, specs_native_ios_player_doc, specs_native_player_doc [EXTRACTED 1.00]
- **iOS lock-screen session-presence plugins (background audio + Live Activity) on the native shell** — specs_background_audio_plugin_doc, specs_live_activity_plugin_doc, specs_native_ios_doc [EXTRACTED 1.00]
- **Shared design-token/utility system (input.css tokens, card family, typography utilities)** — specs_design_system_doc, specs_design_refresh_2026_doc, specs_onboarding_welcome_doc [INFERRED 0.85]

## Communities (1 total, 0 thin omitted)

### Community 0 - "Whole repo (no clustering)"
Cohesion: 1.00
Nodes (4219): $schema, hooks, PostToolUse, pre-push script, xcodebuild, npx, check-prerequisites.sh script, common.sh script (+4211 more)

## Ambiguous Edges - Review These
- `Indigo · Refined — locked light-paper palette` → `Type colour-coding (Piece vs Exercise colour + icon pair)`  [AMBIGUOUS]
  docs/design-principles.md · relation: semantically_similar_to
- `Library Add/Detail/Editing - Tasks` → `LibraryItemView ViewModel`  [AMBIGUOUS]
  specs/_archive/004-library-detail-editing/tasks.md · relation: conceptually_related_to
- `Weekly Practice Summary Requirements Checklist` → `153 Weekly Practice Summary - Spec`  [AMBIGUOUS]
  specs/_archive/153-weekly-practice-summary/checklists/requirements.md · relation: references
- `compute_neglected_items computation approach` → `Decision 5: Lesson as standalone entity, no item relationships yet`  [AMBIGUOUS]
  specs/_archive/153-weekly-practice-summary/research.md · relation: semantically_similar_to

## Knowledge Gaps
- **446 isolated node(s):** `$schema`, `PostToolUse`, `npx`, `check-prerequisites.sh script`, `common.sh script` (+441 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Indigo · Refined — locked light-paper palette` and `Type colour-coding (Piece vs Exercise colour + icon pair)`?**
  _Edge tagged AMBIGUOUS (relation: semantically_similar_to) - confidence is low._
- **What is the exact relationship between `Library Add/Detail/Editing - Tasks` and `LibraryItemView ViewModel`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Weekly Practice Summary Requirements Checklist` and `153 Weekly Practice Summary - Spec`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **What is the exact relationship between `compute_neglected_items computation approach` and `Decision 5: Lesson as standalone entity, no item relationships yet`?**
  _Edge tagged AMBIGUOUS (relation: semantically_similar_to) - confidence is low._
- **What connects `$schema`, `PostToolUse`, `npx` to the rest of the system?**
  _446 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Whole repo (no clustering)` be split into smaller, more focused modules?**
  _Cohesion score 0.0006971243210522333 - nodes in this community are weakly interconnected._