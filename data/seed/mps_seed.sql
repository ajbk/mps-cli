-- MPS clean seed data
-- Canonical MVP data for deterministic class generation.

INSERT INTO movement_experiences (id, name) VALUES
  ('shoulder_freedom', 'Shoulder Freedom'),
  ('happy_hips', 'Happy Hips'),
  ('spine_reset', 'Spine Reset');

INSERT INTO equipment (id, name, category) VALUES
  ('reformer', 'Reformer', 'apparatus'),
  ('chair', 'Wunda Chair', 'apparatus'),
  ('mat', 'Mat', 'movement_context'),
  ('standing', 'Standing', 'movement_context');

INSERT INTO movement_systems (id, name) VALUES
  ('Shoulder', 'Shoulder'),
  ('Thoracic', 'Thoracic'),
  ('Hip', 'Hip'),
  ('Legs', 'Legs'),
  ('Spine', 'Spine'),
  ('BreathCore', 'Breath and Core'),
  ('Balance', 'Balance');

INSERT INTO base_strategies (id, movement_experience, primary_focus, secondary_focus, explanation_template) VALUES
  (1, 'shoulder_freedom', 'Shoulder', 'Thoracic', 'Prioritise shoulder range, scapular control, and thoracic extension so reaching feels easier without rib flare.'),
  (2, 'happy_hips', 'Hip', 'Legs', 'Prioritise hip mobility, glute activation, and leg alignment so squatting and single-leg stance feel clearer.'),
  (3, 'spine_reset', 'Spine', 'BreathCore', 'Prioritise spinal articulation, rotation, and breath-led core support so posture feels decompressed and organised.');

INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES
  (1, 'Shoulder', 40), (1, 'Thoracic', 22), (1, 'BreathCore', 14), (1, 'Spine', 10), (1, 'Hip', 8), (1, 'Legs', 4), (1, 'Balance', 2),
  (2, 'Hip', 36), (2, 'Legs', 24), (2, 'Spine', 14), (2, 'BreathCore', 12), (2, 'Balance', 8), (2, 'Shoulder', 4), (2, 'Thoracic', 2),
  (3, 'Spine', 34), (3, 'BreathCore', 24), (3, 'Thoracic', 18), (3, 'Hip', 10), (3, 'Shoulder', 8), (3, 'Legs', 4), (3, 'Balance', 2);

INSERT INTO base_strategy_objectives (strategy_id, objective) VALUES
  (1, 'Increase overhead reach range'),
  (1, 'Improve scapular stability'),
  (1, 'Release thoracic extension'),
  (2, 'Improve hip flexion and extension range'),
  (2, 'Strengthen glute activation'),
  (2, 'Improve single-leg stability'),
  (3, 'Restore spinal segmental mobility'),
  (3, 'Improve diaphragmatic breathing'),
  (3, 'Reduce spinal compression patterns');

INSERT INTO observation_modifiers (id, explanation_fragment) VALUES
  ('thoracic_stiffness', 'Thoracic stiffness was observed, so thoracic mobility receives extra priority.'),
  ('limited_overhead_reach', 'Limited overhead reach was observed, so shoulder range and rib control are prioritised.'),
  ('desk_posture', 'Desk posture patterns were observed, so the plan counters forward head, rounded shoulders, and shallow breath.'),
  ('hip_tightness', 'Hip tightness was observed, so hip flexor, rotator, and glute work receive extra priority.'),
  ('limited_squat', 'Limited squat depth was observed, so hip mobility and leg alignment receive extra priority.'),
  ('low_back_tension', 'Low-back tension was observed, so decompression and breath-led core support receive extra priority.'),
  ('limited_rotation', 'Limited rotation was observed, so thoracic and spinal rotation receive extra priority.'),
  ('weak_core', 'Weak core connection was observed, so deep stabiliser activation is introduced earlier.'),
  ('balance_concern', 'Balance uncertainty was observed, so balance challenge is introduced gradually.'),
  ('knee_pain', 'Knee sensitivity was reported, so loaded knee flexion is reduced and hip mechanics receive extra priority.'),
  ('wrist_pain', 'Wrist sensitivity was reported, so loaded wrist positions are avoided or regressed.');

INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES
  ('thoracic_stiffness', 'thoracic'), ('thoracic_stiffness', 'stiff'), ('thoracic_stiffness', 'rounded'),
  ('limited_overhead_reach', 'overhead'), ('limited_overhead_reach', 'reach'), ('limited_overhead_reach', 'arm'),
  ('desk_posture', 'desk'), ('desk_posture', 'office'), ('desk_posture', 'sitting'),
  ('hip_tightness', 'hip tight'), ('hip_tightness', 'hip stiffness'), ('hip_tightness', 'tight hips'),
  ('limited_squat', 'squat'), ('limited_squat', 'depth'),
  ('low_back_tension', 'low back'), ('low_back_tension', 'lumbar'), ('low_back_tension', 'back tension'),
  ('limited_rotation', 'rotation'), ('limited_rotation', 'rotate'), ('limited_rotation', 'twist'),
  ('weak_core', 'weak core'), ('weak_core', 'core connection'), ('weak_core', 'stability'),
  ('balance_concern', 'balance'), ('balance_concern', 'unstable'),
  ('knee_pain', 'knee'), ('wrist_pain', 'wrist');

INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES
  ('thoracic_stiffness', 'Thoracic', 7), ('thoracic_stiffness', 'Spine', 3),
  ('limited_overhead_reach', 'Shoulder', 6), ('limited_overhead_reach', 'Thoracic', 3),
  ('desk_posture', 'Thoracic', 5), ('desk_posture', 'Shoulder', 4), ('desk_posture', 'BreathCore', 3),
  ('hip_tightness', 'Hip', 7), ('hip_tightness', 'Legs', -2),
  ('limited_squat', 'Hip', 5), ('limited_squat', 'Legs', 4),
  ('low_back_tension', 'Spine', 6), ('low_back_tension', 'BreathCore', 5),
  ('limited_rotation', 'Thoracic', 5), ('limited_rotation', 'Spine', 5),
  ('weak_core', 'BreathCore', 7), ('weak_core', 'Balance', -2),
  ('balance_concern', 'Balance', 5), ('balance_concern', 'Legs', -2),
  ('knee_pain', 'Legs', -6), ('knee_pain', 'Hip', 4),
  ('wrist_pain', 'Shoulder', -3), ('wrist_pain', 'BreathCore', 2);

INSERT INTO observation_added_objectives (modifier_id, objective) VALUES
  ('desk_posture', 'Counter forward-head and rounded-shoulder posture'),
  ('low_back_tension', 'Create spinal decompression before load'),
  ('weak_core', 'Activate deep stabilisers before larger movement'),
  ('knee_pain', 'Protect knee joints by reducing loaded flexion'),
  ('wrist_pain', 'Avoid sustained weight-bearing through wrists');

INSERT INTO observation_preferred_objectives (modifier_id, objective) VALUES
  ('thoracic_stiffness', 'Release thoracic extension'),
  ('limited_overhead_reach', 'Increase overhead reach range'),
  ('desk_posture', 'Release thoracic extension'),
  ('hip_tightness', 'Improve hip flexion and extension range'),
  ('limited_squat', 'Improve hip flexion and extension range'),
  ('low_back_tension', 'Reduce spinal compression patterns'),
  ('limited_rotation', 'Restore spinal segmental mobility'),
  ('weak_core', 'Improve diaphragmatic breathing'),
  ('balance_concern', 'Improve single-leg stability');

INSERT INTO benchmarks (id, movement_experience, name, instruction) VALUES
  ('overhead_reach', 'shoulder_freedom', 'Overhead Reach Check', 'Stand tall and reach both arms overhead. Notice rib flare, shoulder hiking, and left-right range.'),
  ('wall_slide', 'shoulder_freedom', 'Wall Slide Check', 'Stand with back near a wall and slide arms upward while keeping ribs quiet. Notice where motion changes quality.'),
  ('deep_squat', 'happy_hips', 'Deep Squat Check', 'Lower into a comfortable squat. Notice depth, heel contact, knee tracking, and torso angle.'),
  ('single_leg_stance', 'happy_hips', 'Single-Leg Stance Check', 'Balance on one leg for up to 30 seconds. Notice pelvis level, foot pressure, and sway.'),
  ('standing_roll_down', 'spine_reset', 'Standing Roll Down Check', 'Roll down one segment at a time. Notice where the spine moves freely and where it skips.'),
  ('seated_rotation', 'spine_reset', 'Seated Rotation Check', 'Sit tall and rotate right and left. Notice range, breath, and side-to-side difference.');

INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES
  ('overhead_reach', 'Ribs flare before arms reach vertical'),
  ('overhead_reach', 'Shoulders hike or pinch near the ears'),
  ('overhead_reach', 'One arm arrives later than the other'),
  ('wall_slide', 'Lower back arches to find range'),
  ('wall_slide', 'Elbows or wrists drift away from wall'),
  ('deep_squat', 'Heels lift or knees collapse inward'),
  ('deep_squat', 'Torso falls forward early'),
  ('single_leg_stance', 'Pelvis drops or foot grips hard'),
  ('single_leg_stance', 'Balance cannot settle within 10 seconds'),
  ('standing_roll_down', 'Movement skips through the thoracic or lumbar spine'),
  ('standing_roll_down', 'Breath holds during flexion'),
  ('seated_rotation', 'Hips shift instead of thorax rotating'),
  ('seated_rotation', 'One direction feels blocked or compressed');

INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression) VALUES
  ('footwork', 'Footwork', 'Supine reformer pressing pattern for leg alignment, foot articulation, and breath-supported lower body strength.', 'reformer', 'Beginner', 'IntermediateAdvanced', 2, 7, 'Reduce spring load and range.', 'Add single-leg variations.'),
  ('pelvic_curl', 'Pelvic Curl', 'Supine spinal articulation using the carriage to organise posterior chain and lumbar control.', 'reformer', 'Beginner', 'IntermediateAdvanced', 2, 6, 'Bridge only halfway.', 'Add arm reach or slower tempo.'),
  ('feet_in_straps', 'Feet in Straps', 'Leg arcs and circles on the reformer to open hips while maintaining pelvic stability.', 'reformer', 'BeginnerIntermediate', 'Advanced', 3, 8, 'Use smaller arcs.', 'Add coordination or longer lever.'),
  ('arms_in_straps', 'Arms in Straps', 'Supine arm arcs against straps to train scapular control and rib stability.', 'reformer', 'BeginnerIntermediate', 'Advanced', 3, 7, 'Bend elbows and reduce spring.', 'Add hundred prep pattern.'),
  ('pulling_straps', 'Pulling Straps', 'Prone strap work for thoracic extension, posterior shoulder strength, and postural support.', 'reformer', 'Intermediate', 'Advanced', 4, 6, 'Keep chest low and reduce range.', 'Add T pull or longer hold.'),
  ('short_spine_prep', 'Short Spine Prep', 'Controlled hip and spine articulation with straps for posterior chain length and spinal sequencing.', 'reformer', 'Intermediate', 'Advanced', 4, 7, 'Keep pelvis grounded and use frog pattern.', 'Move toward full short spine.'),
  ('elephant', 'Elephant', 'Standing reformer hinge with carriage movement for posterior chain length, shoulder support, and spinal organisation.', 'reformer', 'BeginnerIntermediate', 'Advanced', 3, 6, 'Bend knees and keep range small.', 'Add single-leg variation.'),
  ('eves_lunge', 'Eve''s Lunge', 'Supported reformer lunge to open hip flexors and connect breath with split-stance control.', 'reformer', 'BeginnerIntermediate', 'Advanced', 3, 6, 'Use hands on frame and smaller range.', 'Add rotation or arm reach.'),
  ('chair_footwork', 'Chair Footwork', 'Seated chair pedal work for lower-body alignment and controlled knee tracking.', 'chair', 'Beginner', 'IntermediateAdvanced', 2, 7, 'Use lighter spring and smaller knee bend.', 'Add heel/toe variations.'),
  ('seated_push_down', 'Seated Push Down', 'Seated chair pedal press to connect shoulder depression, trunk height, and breath.', 'chair', 'Beginner', 'IntermediateAdvanced', 2, 6, 'Use two hands and light spring.', 'Add single-arm version.'),
  ('standing_push_down', 'Standing Push Down', 'Standing single-arm pedal press for shoulder stability and lateral core control.', 'chair', 'BeginnerIntermediate', 'Advanced', 3, 6, 'Use two hands or reduce range.', 'Add contralateral reach.'),
  ('swan_on_chair', 'Swan on Chair', 'Prone chair extension to strengthen thoracic extensors and open the front body.', 'chair', 'BeginnerIntermediate', 'Advanced', 3, 6, 'Keep extension small.', 'Add longer hold or leg reach.'),
  ('mermaid_chair', 'Mermaid on Chair', 'Side-bending chair pattern to open ribs, spine, and shoulder line with breath.', 'chair', 'Beginner', 'IntermediateAdvanced', 2, 6, 'Keep both sit bones grounded.', 'Add rotation.'),
  ('step_up_prep', 'Step Up Prep', 'Supported chair stepping pattern for hip-knee alignment, balance, and functional leg strength.', 'chair', 'Intermediate', 'Advanced', 4, 6, 'Use hands for support and smaller pedal travel.', 'Add arm reach.'),
  ('standing_leg_pump', 'Standing Leg Pump', 'Standing chair pedal press for glute activation, balance, and posterior chain strength.', 'chair', 'Intermediate', 'Advanced', 4, 6, 'Hold chair and reduce spring.', 'Add upright arm pattern.'),
  ('seated_rotation_chair', 'Seated Rotation on Chair', 'Seated rotation and pedal support to organise spine, ribs, and breath.', 'chair', 'BeginnerIntermediate', 'Advanced', 3, 5, 'Reduce rotation range.', 'Add arm arc.'),
  ('breath_reset', 'Breath Reset', 'Supine or seated breathing reset to establish rib expansion, pelvic weight, and attention.', 'mat', 'Beginner', 'Advanced', 1, 4, 'Use supported knees.', 'Add arm float.'),
  ('standing_roll_down_ctx', 'Standing Roll Down', 'Standing spinal articulation used for assessment, preparation, and retest.', 'standing', 'Beginner', 'Advanced', 1, 4, 'Bend knees and reduce range.', 'Slow tempo or close eyes.'),
  ('arm_raise_ctx', 'Arm Raise', 'Standing arm raise to assess overhead reach and teach rib control.', 'standing', 'Beginner', 'Advanced', 1, 3, 'Use smaller range.', 'Add light hand weights.'),
  ('cat_cow', 'Cat-Cow', 'Quadruped or seated spinal flexion and extension for breath-led articulation.', 'mat', 'Beginner', 'Advanced', 1, 5, 'Perform seated on a chair.', 'Add limb reach.'),
  ('standing_rotation_ctx', 'Standing Rotation', 'Standing thoracic rotation pattern for spine mobility and whole-body dissociation.', 'standing', 'BeginnerIntermediate', 'Advanced', 2, 5, 'Reduce range.', 'Add single-leg stance.'),
  ('squat_reach_ctx', 'Squat and Reach', 'Standing squat with reach to integrate hips, legs, shoulders, and breath.', 'standing', 'BeginnerIntermediate', 'Advanced', 2, 5, 'Reduce squat depth.', 'Add rotation at the bottom.'),
  ('single_leg_balance_ctx', 'Single-Leg Balance', 'Standing balance pattern to transfer hip stability into functional stance.', 'standing', 'BeginnerIntermediate', 'Advanced', 2, 4, 'Use wall support.', 'Add head turns.'),
  ('hip_hinge_ctx', 'Hip Hinge', 'Standing hip hinge to teach posterior chain loading and spine protection.', 'standing', 'BeginnerIntermediate', 'Advanced', 2, 5, 'Use dowel feedback.', 'Add single-leg hinge.');

INSERT INTO exercise_roles (exercise_id, role) VALUES
  ('footwork', 'Prepare'), ('footwork', 'Prime'),
  ('pelvic_curl', 'Prepare'), ('pelvic_curl', 'Prime'), ('pelvic_curl', 'Restore'),
  ('feet_in_straps', 'Prime'), ('feet_in_straps', 'Integrate'),
  ('arms_in_straps', 'Prime'), ('arms_in_straps', 'Integrate'),
  ('pulling_straps', 'Prime'), ('pulling_straps', 'Challenge'),
  ('short_spine_prep', 'Integrate'), ('short_spine_prep', 'Restore'),
  ('elephant', 'Prime'), ('elephant', 'Challenge'),
  ('eves_lunge', 'Prepare'), ('eves_lunge', 'Transfer'),
  ('chair_footwork', 'Prepare'), ('chair_footwork', 'Prime'),
  ('seated_push_down', 'Prepare'), ('seated_push_down', 'Prime'),
  ('standing_push_down', 'Prime'), ('standing_push_down', 'Challenge'),
  ('swan_on_chair', 'Prime'), ('swan_on_chair', 'Integrate'),
  ('mermaid_chair', 'Prepare'), ('mermaid_chair', 'Integrate'),
  ('step_up_prep', 'Challenge'), ('step_up_prep', 'Transfer'),
  ('standing_leg_pump', 'Prime'), ('standing_leg_pump', 'Challenge'),
  ('seated_rotation_chair', 'Prime'), ('seated_rotation_chair', 'Integrate'), ('seated_rotation_chair', 'Restore'),
  ('breath_reset', 'Assess'), ('breath_reset', 'Prepare'), ('breath_reset', 'Restore'),
  ('standing_roll_down_ctx', 'Assess'), ('standing_roll_down_ctx', 'Prepare'), ('standing_roll_down_ctx', 'Restore'),
  ('arm_raise_ctx', 'Assess'), ('arm_raise_ctx', 'Prepare'),
  ('cat_cow', 'Assess'), ('cat_cow', 'Prepare'), ('cat_cow', 'Restore'),
  ('standing_rotation_ctx', 'Prepare'), ('standing_rotation_ctx', 'Integrate'), ('standing_rotation_ctx', 'Transfer'),
  ('squat_reach_ctx', 'Integrate'), ('squat_reach_ctx', 'Transfer'),
  ('single_leg_balance_ctx', 'Transfer'),
  ('hip_hinge_ctx', 'Integrate'), ('hip_hinge_ctx', 'Transfer');

INSERT INTO exercise_movement_systems (exercise_id, system_name) VALUES
  ('footwork', 'Legs'), ('footwork', 'Hip'), ('footwork', 'BreathCore'),
  ('pelvic_curl', 'Spine'), ('pelvic_curl', 'Hip'), ('pelvic_curl', 'BreathCore'),
  ('feet_in_straps', 'Hip'), ('feet_in_straps', 'Legs'), ('feet_in_straps', 'BreathCore'),
  ('arms_in_straps', 'Shoulder'), ('arms_in_straps', 'BreathCore'),
  ('pulling_straps', 'Shoulder'), ('pulling_straps', 'Thoracic'), ('pulling_straps', 'Spine'),
  ('short_spine_prep', 'Spine'), ('short_spine_prep', 'Hip'), ('short_spine_prep', 'BreathCore'),
  ('elephant', 'Spine'), ('elephant', 'Shoulder'), ('elephant', 'Legs'),
  ('eves_lunge', 'Hip'), ('eves_lunge', 'Legs'), ('eves_lunge', 'Spine'),
  ('chair_footwork', 'Legs'), ('chair_footwork', 'Hip'),
  ('seated_push_down', 'Shoulder'), ('seated_push_down', 'BreathCore'),
  ('standing_push_down', 'Shoulder'), ('standing_push_down', 'BreathCore'), ('standing_push_down', 'Balance'),
  ('swan_on_chair', 'Spine'), ('swan_on_chair', 'Thoracic'), ('swan_on_chair', 'Shoulder'),
  ('mermaid_chair', 'Spine'), ('mermaid_chair', 'Thoracic'), ('mermaid_chair', 'BreathCore'),
  ('step_up_prep', 'Legs'), ('step_up_prep', 'Hip'), ('step_up_prep', 'Balance'),
  ('standing_leg_pump', 'Hip'), ('standing_leg_pump', 'Legs'), ('standing_leg_pump', 'Balance'),
  ('seated_rotation_chair', 'Spine'), ('seated_rotation_chair', 'Thoracic'), ('seated_rotation_chair', 'BreathCore'),
  ('breath_reset', 'BreathCore'), ('breath_reset', 'Spine'),
  ('standing_roll_down_ctx', 'Spine'), ('standing_roll_down_ctx', 'BreathCore'),
  ('arm_raise_ctx', 'Shoulder'), ('arm_raise_ctx', 'Thoracic'),
  ('cat_cow', 'Spine'), ('cat_cow', 'BreathCore'),
  ('standing_rotation_ctx', 'Thoracic'), ('standing_rotation_ctx', 'Spine'), ('standing_rotation_ctx', 'BreathCore'),
  ('squat_reach_ctx', 'Hip'), ('squat_reach_ctx', 'Legs'), ('squat_reach_ctx', 'Shoulder'),
  ('single_leg_balance_ctx', 'Balance'), ('single_leg_balance_ctx', 'Hip'), ('single_leg_balance_ctx', 'Legs'),
  ('hip_hinge_ctx', 'Hip'), ('hip_hinge_ctx', 'Legs'), ('hip_hinge_ctx', 'Spine');

INSERT INTO exercise_experience_tags (exercise_id, movement_experience) VALUES
  ('footwork', 'happy_hips'), ('footwork', 'spine_reset'),
  ('pelvic_curl', 'spine_reset'), ('pelvic_curl', 'happy_hips'),
  ('feet_in_straps', 'happy_hips'),
  ('arms_in_straps', 'shoulder_freedom'),
  ('pulling_straps', 'shoulder_freedom'), ('pulling_straps', 'spine_reset'),
  ('short_spine_prep', 'spine_reset'), ('short_spine_prep', 'happy_hips'),
  ('elephant', 'spine_reset'), ('elephant', 'shoulder_freedom'),
  ('eves_lunge', 'happy_hips'),
  ('chair_footwork', 'happy_hips'),
  ('seated_push_down', 'shoulder_freedom'),
  ('standing_push_down', 'shoulder_freedom'),
  ('swan_on_chair', 'spine_reset'), ('swan_on_chair', 'shoulder_freedom'),
  ('mermaid_chair', 'spine_reset'), ('mermaid_chair', 'shoulder_freedom'),
  ('step_up_prep', 'happy_hips'),
  ('standing_leg_pump', 'happy_hips'),
  ('seated_rotation_chair', 'spine_reset'),
  ('breath_reset', 'spine_reset'), ('breath_reset', 'shoulder_freedom'), ('breath_reset', 'happy_hips'),
  ('standing_roll_down_ctx', 'spine_reset'), ('standing_roll_down_ctx', 'happy_hips'),
  ('arm_raise_ctx', 'shoulder_freedom'),
  ('cat_cow', 'spine_reset'),
  ('standing_rotation_ctx', 'spine_reset'), ('standing_rotation_ctx', 'shoulder_freedom'),
  ('squat_reach_ctx', 'happy_hips'), ('squat_reach_ctx', 'shoulder_freedom'),
  ('single_leg_balance_ctx', 'happy_hips'),
  ('hip_hinge_ctx', 'happy_hips'), ('hip_hinge_ctx', 'spine_reset');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES
  ('footwork', 'Improve hip flexion and extension range'), ('footwork', 'Strengthen glute activation'), ('footwork', 'Build lower-body alignment'),
  ('pelvic_curl', 'Restore spinal segmental mobility'), ('pelvic_curl', 'Strengthen glute activation'), ('pelvic_curl', 'Reduce spinal compression patterns'),
  ('feet_in_straps', 'Improve hip flexion and extension range'), ('feet_in_straps', 'Improve pelvic control'), ('feet_in_straps', 'Lengthen posterior chain'),
  ('arms_in_straps', 'Improve scapular stability'), ('arms_in_straps', 'Increase overhead reach range'), ('arms_in_straps', 'Improve rib control'),
  ('pulling_straps', 'Release thoracic extension'), ('pulling_straps', 'Improve scapular stability'), ('pulling_straps', 'Strengthen posterior shoulder'),
  ('short_spine_prep', 'Restore spinal segmental mobility'), ('short_spine_prep', 'Improve hip mobility'), ('short_spine_prep', 'Improve diaphragmatic breathing'),
  ('elephant', 'Reduce spinal compression patterns'), ('elephant', 'Release thoracic extension'), ('elephant', 'Lengthen posterior chain'),
  ('eves_lunge', 'Improve hip flexion and extension range'), ('eves_lunge', 'Open hip flexors'), ('eves_lunge', 'Transfer hip extension to standing'),
  ('chair_footwork', 'Build lower-body alignment'), ('chair_footwork', 'Strengthen glute activation'), ('chair_footwork', 'Improve knee tracking'),
  ('seated_push_down', 'Improve scapular stability'), ('seated_push_down', 'Increase overhead reach range'), ('seated_push_down', 'Improve diaphragmatic breathing'),
  ('standing_push_down', 'Improve scapular stability'), ('standing_push_down', 'Improve lateral core control'), ('standing_push_down', 'Challenge standing posture'),
  ('swan_on_chair', 'Release thoracic extension'), ('swan_on_chair', 'Restore spinal segmental mobility'), ('swan_on_chair', 'Open anterior shoulders'),
  ('mermaid_chair', 'Restore spinal segmental mobility'), ('mermaid_chair', 'Improve diaphragmatic breathing'), ('mermaid_chair', 'Release thoracic extension'),
  ('step_up_prep', 'Improve single-leg stability'), ('step_up_prep', 'Strengthen glute activation'), ('step_up_prep', 'Build functional stepping strength'),
  ('standing_leg_pump', 'Strengthen glute activation'), ('standing_leg_pump', 'Improve single-leg stability'), ('standing_leg_pump', 'Build lower-body endurance'),
  ('seated_rotation_chair', 'Restore spinal segmental mobility'), ('seated_rotation_chair', 'Improve diaphragmatic breathing'), ('seated_rotation_chair', 'Release thoracic rotation'),
  ('breath_reset', 'Improve diaphragmatic breathing'), ('breath_reset', 'Reduce spinal compression patterns'), ('breath_reset', 'Settle attention'),
  ('standing_roll_down_ctx', 'Restore spinal segmental mobility'), ('standing_roll_down_ctx', 'Reduce spinal compression patterns'), ('standing_roll_down_ctx', 'Assess posterior chain length'),
  ('arm_raise_ctx', 'Increase overhead reach range'), ('arm_raise_ctx', 'Improve scapular stability'), ('arm_raise_ctx', 'Assess rib compensation'),
  ('cat_cow', 'Restore spinal segmental mobility'), ('cat_cow', 'Improve diaphragmatic breathing'), ('cat_cow', 'Reduce spinal compression patterns'),
  ('standing_rotation_ctx', 'Restore spinal segmental mobility'), ('standing_rotation_ctx', 'Release thoracic rotation'), ('standing_rotation_ctx', 'Integrate upper and lower body'),
  ('squat_reach_ctx', 'Improve hip flexion and extension range'), ('squat_reach_ctx', 'Build lower-body alignment'), ('squat_reach_ctx', 'Increase overhead reach range'),
  ('single_leg_balance_ctx', 'Improve single-leg stability'), ('single_leg_balance_ctx', 'Strengthen glute activation'), ('single_leg_balance_ctx', 'Challenge balance control'),
  ('hip_hinge_ctx', 'Strengthen glute activation'), ('hip_hinge_ctx', 'Reduce spinal compression patterns'), ('hip_hinge_ctx', 'Teach hip-dominant movement');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES
  ('footwork', 'Press through the whole foot and keep knees tracking over toes.', 0), ('footwork', 'Exhale to press, inhale to return with control.', 1),
  ('pelvic_curl', 'Roll one segment at a time and keep the ribs heavy.', 0), ('pelvic_curl', 'Feel hamstrings and glutes support the lift.', 1),
  ('feet_in_straps', 'Move the legs from the hips while the pelvis stays quiet.', 0), ('feet_in_straps', 'Keep the back of the ribs wide on the carriage.', 1),
  ('arms_in_straps', 'Reach from the back of the shoulders, not the neck.', 0), ('arms_in_straps', 'Keep ribs quiet as the arms move.', 1),
  ('pulling_straps', 'Lengthen before lifting the chest.', 0), ('pulling_straps', 'Draw shoulder blades down and wide.', 1),
  ('short_spine_prep', 'Fold at the hips before articulating the spine.', 0), ('short_spine_prep', 'Control the return one segment at a time.', 1),
  ('elephant', 'Lift the ribs away from the wrists.', 0), ('elephant', 'Move the carriage from the hips and breath.', 1),
  ('eves_lunge', 'Reach the back thigh long while keeping the pelvis level.', 0), ('eves_lunge', 'Breathe into the front of the hip.', 1),
  ('chair_footwork', 'Press evenly through both feet and keep the pelvis tall.', 0), ('chair_footwork', 'Control the pedal return without dropping.', 1),
  ('seated_push_down', 'Grow tall as the pedal moves.', 0), ('seated_push_down', 'Keep collarbones wide and neck easy.', 1),
  ('standing_push_down', 'Press from the shoulder blade, not the wrist.', 0), ('standing_push_down', 'Stand tall and keep weight even between feet.', 1),
  ('swan_on_chair', 'Initiate extension from the upper back.', 0), ('swan_on_chair', 'Keep the lower back long.', 1),
  ('mermaid_chair', 'Anchor both sit bones before side bending.', 0), ('mermaid_chair', 'Breathe into the opening side ribs.', 1),
  ('step_up_prep', 'Drive through the standing heel and keep pelvis level.', 0), ('step_up_prep', 'Use the hands only as much as needed.', 1),
  ('standing_leg_pump', 'Press through the heel and keep the torso tall.', 0), ('standing_leg_pump', 'Control the pedal return with the back of the leg.', 1),
  ('seated_rotation_chair', 'Rotate from the ribs while the pelvis stays grounded.', 0), ('seated_rotation_chair', 'Exhale into the turn and inhale back to centre.', 1),
  ('breath_reset', 'Let the inhale widen the side and back ribs.', 0), ('breath_reset', 'Exhale without gripping the throat or jaw.', 1),
  ('standing_roll_down_ctx', 'Nod the head and roll down one segment at a time.', 0), ('standing_roll_down_ctx', 'Soften knees if the back line pulls.', 1),
  ('arm_raise_ctx', 'Reach the fingertips up without flaring the ribs.', 0), ('arm_raise_ctx', 'Keep shoulders broad and neck easy.', 1),
  ('cat_cow', 'Start the movement from the tail and ripple through the spine.', 0), ('cat_cow', 'Match flexion and extension with breath.', 1),
  ('standing_rotation_ctx', 'Rotate from the ribcage and keep hips steady.', 0), ('standing_rotation_ctx', 'Let the eyes lead the turn.', 1),
  ('squat_reach_ctx', 'Sit back into the hips and reach through the arms.', 0), ('squat_reach_ctx', 'Keep knees tracking and chest broad.', 1),
  ('single_leg_balance_ctx', 'Root through the standing foot and level the pelvis.', 0), ('single_leg_balance_ctx', 'Keep the eyes on one steady point.', 1),
  ('hip_hinge_ctx', 'Send hips back while the spine stays long.', 0), ('hip_hinge_ctx', 'Feel hamstrings load before returning upright.', 1);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES
  ('elephant', 'high_blood_pressure', 'HardExclude', 'Avoid loaded inverted hinge for this group.'),
  ('standing_push_down', 'wrist_pain', 'HardExclude', 'Avoid standing pedal press with loaded wrist pressure.'),
  ('cat_cow', 'wrist_pain', 'RequireRegression', 'Use fists, forearms, or seated version.'),
  ('seated_push_down', 'wrist_pain', 'Caution', 'Use lighter spring and neutral wrist.'),
  ('step_up_prep', 'knee_pain', 'RequireRegression', 'Use hand support and reduce pedal travel.'),
  ('chair_footwork', 'knee_pain', 'Caution', 'Use smaller knee bend and lighter spring.'),
  ('standing_leg_pump', 'knee_pain', 'Caution', 'Reduce range and spring resistance.'),
  ('squat_reach_ctx', 'knee_pain', 'Caution', 'Limit squat depth to pain-free range.'),
  ('swan_on_chair', 'low_back_pain', 'Caution', 'Keep extension small and bias upper thoracic movement.'),
  ('hip_hinge_ctx', 'low_back_pain', 'RequireRegression', 'Use dowel feedback and reduce range.'),
  ('single_leg_balance_ctx', 'ankle_instability', 'Caution', 'Use wall support and reduce challenge.');
