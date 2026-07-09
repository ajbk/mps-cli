-- ============================================================
-- MPS Seed Data
-- ============================================================

-- ----------------------------------------------------------
-- Movement Experiences
-- ----------------------------------------------------------
INSERT INTO movement_experiences (id, name) VALUES ('shoulder_freedom', 'Shoulder Freedom');
INSERT INTO movement_experiences (id, name) VALUES ('happy_hips', 'Happy Hips');
INSERT INTO movement_experiences (id, name) VALUES ('spine_reset', 'Spine Reset');

-- ----------------------------------------------------------
-- Equipment
-- ----------------------------------------------------------
INSERT INTO equipment (id, name, category) VALUES ('reformer', 'Reformer', 'apparatus');
INSERT INTO equipment (id, name, category) VALUES ('chair', 'Wunda Chair', 'apparatus');
INSERT INTO equipment (id, name, category) VALUES ('mat', 'Mat', 'movement_context');
INSERT INTO equipment (id, name, category) VALUES ('standing', 'Standing', 'movement_context');

-- ----------------------------------------------------------
-- Movement Systems
-- ----------------------------------------------------------
INSERT INTO movement_systems (id, name) VALUES ('Shoulder', 'Shoulder');
INSERT INTO movement_systems (id, name) VALUES ('Thoracic', 'Thoracic');
INSERT INTO movement_systems (id, name) VALUES ('Hip', 'Hip');
INSERT INTO movement_systems (id, name) VALUES ('Legs', 'Legs');
INSERT INTO movement_systems (id, name) VALUES ('Spine', 'Spine');
INSERT INTO movement_systems (id, name) VALUES ('BreathCore', 'Breath & Core');

-- ----------------------------------------------------------
-- Base Strategies (one per experience)
-- ----------------------------------------------------------

-- Strategy 1: ShoulderFreedom
INSERT INTO base_strategies (id, movement_experience, primary_focus, secondary_focus, explanation_template)
VALUES (1, 'shoulder_freedom', 'Shoulder', 'Thoracic',
        'This session prioritises shoulder range and stability, with secondary attention to thoracic mobility to support overhead and reaching movements.');

INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (1, 'Shoulder', 40);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (1, 'Thoracic', 20);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (1, 'Hip', 15);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (1, 'Legs', 10);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (1, 'Spine', 10);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (1, 'BreathCore', 5);

INSERT INTO base_strategy_objectives (strategy_id, objective) VALUES (1, 'Increase overhead reach range');
INSERT INTO base_strategy_objectives (strategy_id, objective) VALUES (1, 'Improve scapular stability');
INSERT INTO base_strategy_objectives (strategy_id, objective) VALUES (1, 'Release thoracic extension');

-- Strategy 2: HappyHips
INSERT INTO base_strategies (id, movement_experience, primary_focus, secondary_focus, explanation_template)
VALUES (2, 'happy_hips', 'Hip', 'Legs',
        'This session focuses on hip mobility and freedom, with secondary attention to leg strength and alignment to support functional movement.');

INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (2, 'Hip', 35);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (2, 'Legs', 25);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (2, 'Spine', 20);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (2, 'BreathCore', 12);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (2, 'Shoulder', 5);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (2, 'Thoracic', 3);

INSERT INTO base_strategy_objectives (strategy_id, objective) VALUES (2, 'Improve hip flexion and extension range');
INSERT INTO base_strategy_objectives (strategy_id, objective) VALUES (2, 'Strengthen glute activation');
INSERT INTO base_strategy_objectives (strategy_id, objective) VALUES (2, 'Improve single-leg stability');

-- Strategy 3: SpineReset
INSERT INTO base_strategies (id, movement_experience, primary_focus, secondary_focus, explanation_template)
VALUES (3, 'spine_reset', 'Spine', 'BreathCore',
        'This session restores spinal mobility and breath-pattern integration, with secondary focus on core support for healthy posture.');

INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (3, 'Spine', 30);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (3, 'BreathCore', 25);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (3, 'Thoracic', 20);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (3, 'Shoulder', 15);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (3, 'Hip', 5);
INSERT INTO base_strategy_emphasis (strategy_id, system_name, emphasis_value) VALUES (3, 'Legs', 5);

INSERT INTO base_strategy_objectives (strategy_id, objective) VALUES (3, 'Restore spinal segmental mobility');
INSERT INTO base_strategy_objectives (strategy_id, objective) VALUES (3, 'Improve diaphragmatic breathing');
INSERT INTO base_strategy_objectives (strategy_id, objective) VALUES (3, 'Reduce spinal compression patterns');

-- ----------------------------------------------------------
-- Observation Modifiers (12 modifiers)
-- ----------------------------------------------------------

INSERT INTO observation_modifiers (id, explanation_fragment) VALUES ('thoracic_stiffness', 'Thoracic stiffness detected — increasing thoracic mobilisation work.');
INSERT INTO observation_modifiers (id, explanation_fragment) VALUES ('office_syndrome', 'Desk/office patterns observed — adding shoulder and thoracic counter-movements.');
INSERT INTO observation_modifiers (id, explanation_fragment) VALUES ('hip_tightness', 'Hip tightness detected — prioritising hip flexor and rotator releases.');
INSERT INTO observation_modifiers (id, explanation_fragment) VALUES ('limited_squat', 'Limited squat depth observed — adding hip and leg mobility work.');
INSERT INTO observation_modifiers (id, explanation_fragment) VALUES ('low_back_tension', 'Low back tension detected — adding spinal decompression and core stabilisation.');
INSERT INTO observation_modifiers (id, explanation_fragment) VALUES ('limited_rotation', 'Limited rotational range observed — adding thoracic and spinal rotation drills.');
INSERT INTO observation_modifiers (id, explanation_fragment) VALUES ('weak_core', 'Core weakness detected — increasing core activation and stabilisation exercises.');
INSERT INTO observation_modifiers (id, explanation_fragment) VALUES ('limited_overhead_reach', 'Limited overhead reach observed — adding shoulder and thoracic extension work.');
INSERT INTO observation_modifiers (id, explanation_fragment) VALUES ('knee_pain', 'Knee discomfort reported — regressing lower-body load and focusing on hip mechanics.');
INSERT INTO observation_modifiers (id, explanation_fragment) VALUES ('wrist_pain', 'Wrist discomfort reported — reducing weight-bearing on hands.');
INSERT INTO observation_modifiers (id, explanation_fragment) VALUES ('balance_concern', 'Balance concerns observed — adding balance-challenge progressions carefully.');
INSERT INTO observation_modifiers (id, explanation_fragment) VALUES ('upper_cross', 'Upper-cross pattern detected — addressing thoracic extension and shoulder positioning.');

-- Keywords for each modifier
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('thoracic_stiffness', 'thoracic');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('thoracic_stiffness', 'stiff');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('thoracic_stiffness', 'rounded');

INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('office_syndrome', 'office');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('office_syndrome', 'desk');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('office_syndrome', 'sitting');

INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('hip_tightness', 'hip');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('hip_tightness', 'tight');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('hip_tightness', 'stiff hip');

INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('limited_squat', 'squat');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('limited_squat', 'depth');

INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('low_back_tension', 'low back');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('low_back_tension', 'lumbar');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('low_back_tension', 'back pain');

INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('limited_rotation', 'rotation');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('limited_rotation', 'twist');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('limited_rotation', 'rotate');

INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('weak_core', 'core');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('weak_core', 'weak');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('weak_core', 'stability');

INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('limited_overhead_reach', 'overhead');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('limited_overhead_reach', 'reach');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('limited_overhead_reach', 'arm');

INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('knee_pain', 'knee');

INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('wrist_pain', 'wrist');

INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('balance_concern', 'balance');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('balance_concern', 'unstable');

INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('upper_cross', 'upper cross');
INSERT INTO observation_modifier_keywords (modifier_id, keyword) VALUES ('upper_cross', 'rounded shoulders');

-- Emphasis adjustments for each modifier
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('thoracic_stiffness', 'Thoracic', 5);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('thoracic_stiffness', 'Shoulder', -3);

INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('office_syndrome', 'Shoulder', 3);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('office_syndrome', 'Thoracic', 2);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('office_syndrome', 'BreathCore', 2);

INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('hip_tightness', 'Hip', 5);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('hip_tightness', 'Legs', -3);

INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('limited_squat', 'Hip', 5);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('limited_squat', 'Legs', 3);

INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('low_back_tension', 'Spine', 5);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('low_back_tension', 'BreathCore', 3);

INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('limited_rotation', 'Spine', 5);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('limited_rotation', 'Thoracic', 3);

INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('weak_core', 'BreathCore', 5);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('weak_core', 'Balance', -2);

INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('limited_overhead_reach', 'Shoulder', 3);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('limited_overhead_reach', 'Thoracic', 2);

INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('knee_pain', 'Legs', -5);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('knee_pain', 'Hip', 3);

INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('wrist_pain', 'Shoulder', -3);

INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('balance_concern', 'Balance', 5);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('balance_concern', 'Legs', -3);

INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('upper_cross', 'Thoracic', 5);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('upper_cross', 'Shoulder', 3);
INSERT INTO observation_emphasis_adjustments (modifier_id, system_name, adjustment) VALUES ('upper_cross', 'Legs', -2);

-- Added objectives for select modifiers
INSERT INTO observation_added_objectives (modifier_id, objective) VALUES ('low_back_tension', 'Reduce lumbar compression through spinal elongation');
INSERT INTO observation_added_objectives (modifier_id, objective) VALUES ('knee_pain', 'Protect knee joint by reducing loaded flexion');
INSERT INTO observation_added_objectives (modifier_id, objective) VALUES ('wrist_pain', 'Avoid aggravating wrist with weight-bearing exercises');
INSERT INTO observation_added_objectives (modifier_id, objective) VALUES ('weak_core', 'Activate deep stabilisers before load');
INSERT INTO observation_added_objectives (modifier_id, objective) VALUES ('office_syndrome', 'Counteract forward-head and rounded-shoulder posture');

-- Preferred objectives (objectives to prioritise from base strategy)
INSERT INTO observation_preferred_objectives (modifier_id, objective) VALUES ('thoracic_stiffness', 'Release thoracic extension');
INSERT INTO observation_preferred_objectives (modifier_id, objective) VALUES ('office_syndrome', 'Release thoracic extension');
INSERT INTO observation_preferred_objectives (modifier_id, objective) VALUES ('hip_tightness', 'Improve hip flexion and extension range');
INSERT INTO observation_preferred_objectives (modifier_id, objective) VALUES ('limited_squat', 'Improve hip flexion and extension range');
INSERT INTO observation_preferred_objectives (modifier_id, objective) VALUES ('low_back_tension', 'Reduce spinal compression patterns');
INSERT INTO observation_preferred_objectives (modifier_id, objective) VALUES ('limited_rotation', 'Restore spinal segmental mobility');
INSERT INTO observation_preferred_objectives (modifier_id, objective) VALUES ('limited_overhead_reach', 'Increase overhead reach range');
INSERT INTO observation_preferred_objectives (modifier_id, objective) VALUES ('balance_concern', 'Improve single-leg stability');
INSERT INTO observation_preferred_objectives (modifier_id, objective) VALUES ('upper_cross', 'Release thoracic extension');
INSERT INTO observation_preferred_objectives (modifier_id, objective) VALUES ('upper_cross', 'Increase overhead reach range');

-- ----------------------------------------------------------
-- Benchmarks
-- ----------------------------------------------------------

-- ShoulderFreedom benchmarks
INSERT INTO benchmarks (id, movement_experience, name, instruction) VALUES
    ('overhead_reach_test', 'shoulder_freedom', 'Overhead Reach Test',
     'Stand tall and slowly reach both arms overhead. Observe how far the arms travel before the ribs flare or the lower back arches. Note symmetry between left and right.');

INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('overhead_reach_test', 'Rib flare before arms reach vertical');
INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('overhead_reach_test', 'Asymmetry between left and right arm path');
INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('overhead_reach_test', 'Lower back arch compensation');

INSERT INTO benchmarks (id, movement_experience, name, instruction) VALUES
    ('shoulder_rotation_check', 'shoulder_freedom', 'Shoulder Rotation Check',
     'Stand with arms at 90 degrees of abduction, elbows bent to 90 degrees. Rotate the arms outward (external rotation) and inward (internal rotation). Observe range and quality of movement.');

INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('shoulder_rotation_check', 'Limited external rotation compared to internal rotation');
INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('shoulder_rotation_check', 'Shoulder hiking during rotation');
INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('shoulder_rotation_check', 'Pain or clicking during arc of motion');

-- HappyHips benchmarks
INSERT INTO benchmarks (id, movement_experience, name, instruction) VALUES
    ('deep_squat_test', 'happy_hips', 'Deep Squat Test',
     'With feet hip-width apart, lower into a deep squat. Heels should stay on the floor. Observe depth, torso position, and knee tracking relative to toes.');

INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('deep_squat_test', 'Heels lifting off the floor');
INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('deep_squat_test', 'Excessive forward lean of the torso');
INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('deep_squat_test', 'Knees caving inward past big toe');

INSERT INTO benchmarks (id, movement_experience, name, instruction) VALUES
    ('single_leg_balance', 'happy_hips', 'Single Leg Balance',
     'Stand on one leg with the opposite hip at 90 degrees of flexion. Hold for 30 seconds. Observe pelvic stability, hip drop, and sway.');

INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('single_leg_balance', 'Pelvic drop on the unsupported side');
INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('single_leg_balance', 'Excessive lateral sway');
INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('single_leg_balance', 'Unable to hold for 10 seconds without touching down');

-- SpineReset benchmarks
INSERT INTO benchmarks (id, movement_experience, name, instruction) VALUES
    ('seated_rotation_test', 'spine_reset', 'Seated Rotation Test',
     'Sit tall on a chair with arms crossed over the chest. Rotate the torso to the right and left. Observe range and where rotation initiates (upper vs lower spine).');

INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('seated_rotation_test', 'Rotation limited to less than 45 degrees each side');
INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('seated_rotation_test', 'Asymmetry between left and right rotation');
INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('seated_rotation_test', 'Hips rotating with the torso rather than isolating thoracic movement');

INSERT INTO benchmarks (id, movement_experience, name, instruction) VALUES
    ('standing_roll_down', 'spine_reset', 'Standing Roll Down',
     'Stand with feet hip-width apart and slowly roll down through the spine one vertebra at a time. Observe segmental movement quality and where flexion is limited.');

INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('standing_roll_down', 'Stiff mid-thoracic region — difficulty initiating roll');
INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('standing_roll_down', 'Hamstring tightness pulling into forward fold too early');
INSERT INTO benchmark_watch_points (benchmark_id, watch_point) VALUES ('standing_roll_down', 'Asymmetric spinal curve during descent');

-- ----------------------------------------------------------
-- Exercises (24 exercises)
-- ----------------------------------------------------------

-- ===== REFORMER (10) =====

-- 1. Footwork
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('footwork', 'Footwork',
        'Lying on the reformer, press the carriage away using the feet in various positions — toes, arches, heels — to build lower-body strength and alignment.',
        'reformer', 'Beginner', 'IntermediateAdvanced', 2, 8,
        NULL, 'Increase spring resistance or add single-leg variations');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('footwork', 'Prepare');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('footwork', 'Prime');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('footwork', 'Activate glutes and hamstrings');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('footwork', 'Improve ankle and foot articulation');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('footwork', 'Establish lower-body alignment patterns');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('footwork', 'Press through the whole foot — feel all ten toes', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('footwork', 'Keep the pelvis neutral as the carriage moves', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('footwork', 'Knees track over the second and third toes', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('footwork', 'knee_pain', 'Caution', 'Reduce range of motion or spring load if knee discomfort occurs');

-- 2. Pelvic Curl
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('pelvic_curl', 'Pelvic Curl',
        'Lying supine on the reformer with feet on the footbar, articulate the spine off the mat one vertebra at a time into a bridge, then roll back down sequentially.',
        'reformer', 'Beginner', 'IntermediateAdvanced', 1, 6,
        NULL, 'Add single-leg bridge or increase spring tension');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('pelvic_curl', 'Prepare');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('pelvic_curl', 'Prime');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('pelvic_curl', 'Mobilise spinal segments through articulation');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('pelvic_curl', 'Activate posterior chain');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('pelvic_curl', 'Improve breath coordination with movement');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('pelvic_curl', 'Peel the spine off the mat one vertebra at a time', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('pelvic_curl', 'Exhale to roll up, inhale at the top, exhale to roll down', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('pelvic_curl', 'Press evenly through both feet — avoid dominance on one side', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('pelvic_curl', 'low_back_pain', 'Caution', 'Limit bridge height if lumbar discomfort is present');

-- 3. Feet in Straps
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('feet_in_straps', 'Feet in Straps',
        'Lying supine, place the feet in long straps and trace arcs of motion in the hips — circles, V-shapes, and frog patterns — to mobilise the hip joint under load.',
        'reformer', 'BeginnerIntermediate', 'Advanced', 3, 8,
        'Reduce strap range or use lighter springs', 'Increase range of motion or slow down the tempo');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('feet_in_straps', 'Prime');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('feet_in_straps', 'Integrate');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('feet_in_straps', 'Increase hip range of motion in multiple planes');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('feet_in_straps', 'Challenge pelvic stability during dynamic leg movement');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('feet_in_straps', 'Keep the pelvis anchored to the mat as the legs move', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('feet_in_straps', 'Move from the hip joint — avoid gripping the lower back', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('feet_in_straps', 'Maintain even spring tension — control the return', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('feet_in_straps', 'hip_pain', 'Caution', 'Reduce range if hip impingement symptoms appear');

-- 4. Arms in Straps
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('arms_in_straps', 'Arms in Straps',
        'Lying supine with arms in long straps, perform various arm arcs — down-press, chest expansion, arm circles — to build shoulder stability and thoracic awareness.',
        'reformer', 'BeginnerIntermediate', 'Advanced', 3, 8,
        'Use lighter springs and reduce arc range', 'Add larger arcs or slower tempos');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('arms_in_straps', 'Prime');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('arms_in_straps', 'Integrate');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('arms_in_straps', 'Build scapular stability in open chain');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('arms_in_straps', 'Increase shoulder flexion and extension range');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('arms_in_straps', 'Improve breath patterning under load');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('arms_in_straps', 'Draw shoulder blades down and wide before initiating', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('arms_in_straps', 'Exhale to pull — the arms follow the breath', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('arms_in_straps', 'Keep the ribcage heavy on the mat — avoid arching', 2);

-- 5. Pulling Straps
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('pulling_straps', 'Pulling Straps',
        'Prone on the reformer with arms holding long straps, pull the arms toward the hips to extend the upper back and strengthen posterior shoulder muscles.',
        'reformer', 'Intermediate', 'Advanced', 3, 6,
        'Reduce range of motion or spring tension', 'Add full T-pull or overhead pull variations');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('pulling_straps', 'Prime');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('pulling_straps', 'Challenge');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('pulling_straps', 'Strengthen posterior shoulder and upper-back extensors');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('pulling_straps', 'Improve thoracic extension');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('pulling_straps', 'Lengthen the spine before pulling — reach the arms long', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('pulling_straps', 'Pull from the upper back, not the arms alone', 1);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('pulling_straps', 'low_back_pain', 'Caution', 'Limit extension range and ensure core engagement');

-- 6. Short Spine Prep
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('short_spine_prep', 'Short Spine Prep',
        'Lying supine with feet in straps, curl the hips off the mat and roll the spine up toward the shoulders, then articulate back down. Mobilises the spine through flexion.',
        'reformer', 'BeginnerIntermediate', 'Advanced', 3, 6,
        'Use lighter springs and limit roll height', 'Full short spine massage with inversion');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('short_spine_prep', 'Integrate');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('short_spine_prep', 'Restore');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('short_spine_prep', 'Mobilise the spine through sequential flexion');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('short_spine_prep', 'Decompress the lumbar spine');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('short_spine_prep', 'Improve breath-movement coordination');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('short_spine_prep', 'Let the springs help lift the legs — avoid using momentum', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('short_spine_prep', 'Roll down slowly, placing one vertebra at a time back to the mat', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('short_spine_prep', 'Keep the shoulders soft — avoid pressing into the mat with the neck', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('short_spine_prep', 'low_back_pain', 'RequireRegression', 'Reduce inversion angle or substitute Pelvic Curl if lumbar pain is present');
INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('short_spine_prep', 'neck_pain', 'Caution', 'Ensure adequate padding and avoid excessive cervical load');

-- 7. Elephant
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('elephant', 'Elephant',
        'Hands on the footbar and feet on the shoulder blocks in an inverted V position. Articulate the spine to push the carriage back with the legs, then draw it in.',
        'reformer', 'Intermediate', 'Advanced', 4, 6,
        'Reduce spring tension or limit carriage travel', 'Add single-leg elephant or slow tempo holds');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('elephant', 'Prime');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('elephant', 'Challenge');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('elephant', 'Strengthen core in closed-chain flexion');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('elephant', 'Mobilise hamstrings and posterior chain');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('elephant', 'Build wrist and shoulder load tolerance');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('elephant', 'Press the heels into the shoulder blocks to initiate carriage movement', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('elephant', 'Articulate from the pelvis — the spine follows the tailbone', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('elephant', 'Spread the fingers wide to distribute weight through the hands', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('elephant', 'wrist_pain', 'RequireRegression', 'Substitute a non-weight-bearing exercise or use fist support');
INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('elephant', 'high_blood_pressure', 'HardExclude', 'Inverted position is contraindicated');

-- 8. Long Stretch Prep
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('long_stretch_prep', 'Long Stretch Prep',
        'A plank position with hands on the footbar and feet on the shoulder blocks. Push the carriage back and pull it in using total-body coordination and control.',
        'reformer', 'IntermediateAdvanced', 'Advanced', 4, 6,
        'Use heavier springs to assist carriage return', 'Add leg lifts or walking plank variations');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('long_stretch_prep', 'Prime');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('long_stretch_prep', 'Challenge');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('long_stretch_prep', 'Challenge total-body plank integration');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('long_stretch_prep', 'Build shoulder stabilisation under load');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('long_stretch_prep', 'Improve core endurance');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('long_stretch_prep', 'Create one long line from head to heels', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('long_stretch_prep', 'Push the carriage back from the shoulders, not the hands', 1);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('long_stretch_prep', 'wrist_pain', 'RequireRegression', 'Use forearm plank variation or substitute');
INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('long_stretch_prep', 'shoulder_instability', 'Caution', 'Reduce range and ensure scapular control');

-- 9. Eve's Lunge
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('eves_lunge', 'Eve''s Lunge',
        'Standing on the reformer with one foot on the carriage and one on the frame, perform a deep lunge to mobilise the hip flexor and build single-leg strength.',
        'reformer', 'BeginnerIntermediate', 'Advanced', 3, 6,
        'Use hands on the reformer for support', 'Add arm reaches or torso rotation during lunge');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('eves_lunge', 'Prime');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('eves_lunge', 'Integrate');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('eves_lunge', 'Lengthen hip flexors in an extended position');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('eves_lunge', 'Build single-leg stability and strength');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('eves_lunge', 'Improve balance in dynamic stance');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('eves_lunge', 'Drop straight down — avoid letting the front knee drift forward past the toes', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('eves_lunge', 'Keep the pelvis level and square to the front', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('eves_lunge', 'Breathe into the back of the hip of the back leg', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('eves_lunge', 'knee_pain', 'Caution', 'Reduce lunge depth or use hands for partial weight support');

-- 10. Mermaid (Reformer)
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('mermaid_reformer', 'Mermaid',
        'Sitting sideways on the reformer with one hand on the footbar, press the carriage away while laterally flexing the torso. Opens the side body and mobilises the thoracolumbar fascia.',
        'reformer', 'Beginner', 'IntermediateAdvanced', 2, 6,
        NULL, 'Add arm variations or increase lateral range');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('mermaid_reformer', 'Prepare');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('mermaid_reformer', 'Restore');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('mermaid_reformer', 'Mobilise lateral spinal flexion');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('mermaid_reformer', 'Open the side body and intercostals');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('mermaid_reformer', 'Promote breath expansion into the side ribs');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('mermaid_reformer', 'Lengthen both sides of the waist — avoid collapsing into the bottom side', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('mermaid_reformer', 'Press the carriage away from the shoulder, not the hand', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('mermaid_reformer', 'Inhale to lengthen, exhale to deepen the side bend', 2);

-- ===== CHAIR (6) =====

-- 11. Seated Push Down
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('seated_push_down', 'Seated Push Down',
        'Sitting tall on the chair, press both pedals down using the arms to activate the shoulder girdle and build overhead stability. Can be performed bilateral or single arm.',
        'chair', 'BeginnerIntermediate', 'Advanced', 3, 6,
        'Use lighter pedal resistance', 'Add single-arm press or overhead reach at top');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('seated_push_down', 'Prime');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('seated_push_down', 'Challenge');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('seated_push_down', 'Build overhead pressing strength');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('seated_push_down', 'Challenge seated postural control under load');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('seated_push_down', 'Improve scapular upward rotation');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('seated_push_down', 'Sit tall — lengthen the spine before pressing', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('seated_push_down', 'Press straight down — avoid leaning forward', 1);

-- 12. Standing Push Down
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('standing_push_down', 'Standing Push Down',
        'Standing beside the chair, press the pedal down using one arm while maintaining upright posture. Challenges shoulder stability and single-side core engagement.',
        'chair', 'Intermediate', 'Advanced', 4, 6,
        'Use lighter resistance or hold with two hands', 'Add overhead reach or lunge variation');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('standing_push_down', 'Prime');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('standing_push_down', 'Challenge');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_push_down', 'Stabilise the shoulder in standing closed-chain');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_push_down', 'Challenge lateral core stability');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_push_down', 'Build functional pressing pattern');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_push_down', 'Keep the standing side long — avoid collapsing into the hip', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_push_down', 'Press through the whole hand, evenly distributing weight', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_push_down', 'Breathe out as you press — maintain postural height', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('standing_push_down', 'wrist_pain', 'Caution', 'Modify hand position or reduce resistance');

-- 13. Swan on Chair
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('swan_on_chair', 'Swan on Chair',
        'Lying prone over the chair with hands on the pedal, press down to extend the upper back and open the chest. Strengthens posterior chain and improves thoracic extension.',
        'chair', 'BeginnerIntermediate', 'Advanced', 3, 6,
        'Reduce range of extension', 'Add leg lifts or increase extension range');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('swan_on_chair', 'Prime');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('swan_on_chair', 'Integrate');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('swan_on_chair', 'Strengthen thoracic extensors');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('swan_on_chair', 'Open the chest and anterior shoulders');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('swan_on_chair', 'Mobilise spinal extension');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('swan_on_chair', 'Lengthen before you lift — reach the crown of the head away', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('swan_on_chair', 'Initiate from the upper back, not the lower back', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('swan_on_chair', 'Keep the legs engaged — reach the toes away as you lift', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('swan_on_chair', 'low_back_pain', 'Caution', 'Limit extension range and focus on upper thoracic initiation');

-- 14. Mermaid on Chair
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('mermaid_chair', 'Mermaid on Chair',
        'Sitting sideways on the chair with one hand on the pedal, press down to laterally flex the torso. Opens the side body and integrates breath with lateral movement.',
        'chair', 'Beginner', 'IntermediateAdvanced', 2, 6,
        NULL, 'Add arm reaches or combine with rotation');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('mermaid_chair', 'Prepare');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('mermaid_chair', 'Integrate');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('mermaid_chair', 'Mobilise lateral flexion of the spine');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('mermaid_chair', 'Integrate breath with side-body opening');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('mermaid_chair', 'Stabilise pelvis during lateral movement');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('mermaid_chair', 'Sit evenly on both sit bones before you begin', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('mermaid_chair', 'Press the pedal with the shoulder, not the wrist', 1);

-- 15. Step Up Prep
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('step_up_prep', 'Step Up Prep',
        'Standing beside the chair, step onto the pedal and lower the opposite foot to the floor. Builds single-leg strength, balance, and functional stepping patterns.',
        'chair', 'IntermediateAdvanced', 'Advanced', 4, 6,
        'Use hands on the chair for support', 'Add arm reaches or close eyes for balance challenge');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('step_up_prep', 'Challenge');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('step_up_prep', 'Transfer');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('step_up_prep', 'Build single-leg strength for functional activities');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('step_up_prep', 'Challenge balance in dynamic stance');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('step_up_prep', 'Strengthen hip and knee stabilisers');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('step_up_prep', 'Drive through the standing leg — avoid pushing off the toes of the trailing leg', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('step_up_prep', 'Keep the pelvis level as you step up and down', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('step_up_prep', 'Control the pedal return — don''t drop down', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('step_up_prep', 'knee_pain', 'Caution', 'Reduce step height and use hand support');

-- 16. Standing Leg Pump
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('standing_leg_pump', 'Standing Leg Pump',
        'Standing behind the chair, place one foot on the pedal and press down while maintaining upright posture. Builds hip and leg strength in a functional standing position.',
        'chair', 'Intermediate', 'Advanced', 4, 6,
        'Use hands on the chair for balance', 'Add arm movements or single-arm reach during pump');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('standing_leg_pump', 'Prime');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('standing_leg_pump', 'Challenge');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_leg_pump', 'Strengthen glutes and hamstrings in standing');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_leg_pump', 'Improve standing balance under load');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_leg_pump', 'Build functional lower-body endurance');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_leg_pump', 'Press the pedal down through the heel — feel the glute activate', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_leg_pump', 'Stand tall — avoid leaning onto the chair', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_leg_pump', 'Breathe out on the press, inhale on the return', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('standing_leg_pump', 'knee_pain', 'Caution', 'Reduce range and spring resistance');

-- ===== MAT / STANDING (8) =====

-- 17. Standing Roll Down
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('standing_roll_down', 'Standing Roll Down',
        'Standing with feet hip-width apart, articulate the spine forward one vertebra at a time, letting the head and arms hang, then roll back up sequentially. An assessment and warm-up staple.',
        'mat', 'Beginner', 'Advanced', 1, 4,
        NULL, 'Close eyes or slow the tempo for proprioceptive challenge');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('standing_roll_down', 'Assess');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('standing_roll_down', 'Prepare');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('standing_roll_down', 'Restore');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_roll_down', 'Assess spinal segmental mobility');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_roll_down', 'Mobilise the spine through flexion');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_roll_down', 'Release tension in the posterior chain');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_roll_down', 'Nod the chin and let the head lead the descent', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_roll_down', 'Bend the knees slightly if the hamstrings limit the movement', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_roll_down', 'Roll up from the base of the spine — stack each vertebra', 2);

-- 18. Arm Raise
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('arm_raise', 'Arm Raise',
        'Standing tall, float the arms overhead in the frontal or sagittal plane while maintaining spinal alignment. An assessment of shoulder range and a gentle warm-up.',
        'mat', 'Beginner', 'Advanced', 1, 3,
        NULL, 'Add light weights or slow the tempo');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('arm_raise', 'Assess');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('arm_raise', 'Prepare');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('arm_raise', 'Assess overhead reach range');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('arm_raise', 'Activate shoulder stabilisers gently');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('arm_raise', 'Establish breath-arm coordination');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('arm_raise', 'Inhale to float the arms up — exhale to lower', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('arm_raise', 'Keep the ribs knitted together — avoid arching the lower back', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('arm_raise', 'Reach through the fingertips — lengthen before you lift', 2);

-- 19. Standing Rotation
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('standing_rotation', 'Standing Rotation',
        'Standing with arms extended, rotate the torso from side to side while keeping the hips facing forward. Mobilises thoracic rotation and integrates breath.',
        'mat', 'BeginnerIntermediate', 'Advanced', 2, 5,
        'Reduce range of motion', 'Add arm reach patterns or single-leg stance');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('standing_rotation', 'Prepare');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('standing_rotation', 'Integrate');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_rotation', 'Mobilise thoracic rotation');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_rotation', 'Integrate upper and lower body dissociation');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_rotation', 'Rotate from the ribcage — keep the hips and knees still', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_rotation', 'Exhale as you rotate — feel the obliques engage', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_rotation', 'Lead with the eyes — look in the direction of rotation', 2);

-- 20. Squat + Reach
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('squat_reach', 'Squat + Reach',
        'Standing with feet hip-width apart, lower into a squat while reaching the arms forward or overhead. Combines lower-body strength with upper-body mobility.',
        'mat', 'BeginnerIntermediate', 'Advanced', 2, 6,
        'Reduce squat depth or use wall support', 'Add rotation at the bottom or single-leg variation');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('squat_reach', 'Prime');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('squat_reach', 'Integrate');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('squat_reach', 'Build functional squat pattern');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('squat_reach', 'Integrate upper and lower body movement');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('squat_reach', 'Challenge balance in dynamic descent');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('squat_reach', 'Sit back into the hips — weight in the heels', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('squat_reach', 'Reach the arms as the hips descend — counterbalance', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('squat_reach', 'Keep the chest lifted — avoid rounding the upper back', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('squat_reach', 'knee_pain', 'Caution', 'Limit squat depth to pain-free range');

-- 21. Single Leg Balance
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('single_leg_balance', 'Single Leg Balance',
        'Standing on one leg, maintain balance while the free leg performs various movements — toe taps, hip circles, or reaches. Challenges hip and ankle stabilisers.',
        'mat', 'Intermediate', 'Advanced', 3, 5,
        'Hold onto a wall or chair for support', 'Close eyes, add arm movements, or use unstable surface');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('single_leg_balance', 'Challenge');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('single_leg_balance', 'Transfer');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('single_leg_balance', 'Challenge single-leg standing balance');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('single_leg_balance', 'Activate hip stabilisers in functional stance');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('single_leg_balance', 'Improve proprioception');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('single_leg_balance', 'Root through the standing foot — spread the toes', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('single_leg_balance', 'Keep the standing hip level — avoid hiking', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('single_leg_balance', 'Fix the eyes on a steady point to help balance', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('single_leg_balance', 'ankle_instability', 'Caution', 'Use wall support or reduce challenge level');

-- 22. Hip Hinge
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('hip_hinge', 'Hip Hinge',
        'Standing with feet hip-width apart, hinge at the hips while keeping the spine neutral. Builds posterior chain strength and teaches safe bending mechanics.',
        'mat', 'BeginnerIntermediate', 'Advanced', 2, 5,
        'Use a dowel along the spine for feedback', 'Add single-leg hinge or external resistance');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('hip_hinge', 'Prime');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('hip_hinge', 'Integrate');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('hip_hinge', 'Teach hip-dominant movement pattern');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('hip_hinge', 'Strengthen glutes and hamstrings');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('hip_hinge', 'Protect the spine by learning to hinge, not flex');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('hip_hinge', 'Push the hips back — imagine closing a car door with the tailbone', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('hip_hinge', 'Keep a long spine — from head to tailbone is one solid line', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('hip_hinge', 'Feel the hamstrings load as the torso tilts forward', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('hip_hinge', 'low_back_pain', 'Caution', 'Reduce range and use dowel for proprioceptive feedback');

-- 23. Standing Reach
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('standing_reach', 'Standing Reach',
        'Standing tall, reach one or both arms overhead or diagonally while maintaining spinal alignment. Opens the shoulders and integrates breath with reaching patterns.',
        'standing', 'Beginner', 'IntermediateAdvanced', 2, 4,
        NULL, 'Add rotation or combine with step patterns');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('standing_reach', 'Prepare');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('standing_reach', 'Integrate');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_reach', 'Increase overhead reach range in standing');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('standing_reach', 'Integrate breath with upper-body movement');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_reach', 'Reach through the fingertips — imagine touching the ceiling', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_reach', 'Keep the ribs connected — avoid flaring as you reach', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('standing_reach', 'Breathe into the sides and back of the ribcage', 2);

-- 24. Cat-Cow
INSERT INTO exercises (id, name, description, equipment, min_level, max_level, difficulty, default_duration_minutes, regression, progression)
VALUES ('cat_cow', 'Cat-Cow',
        'On hands and knees, alternate between spinal flexion (cat) and extension (cow) to mobilise the full spine and integrate breath with segmental movement.',
        'mat', 'Beginner', 'Advanced', 1, 5,
        'Perform seated on a chair instead', 'Slow the tempo or add limb reaches in each position');

INSERT INTO exercise_roles (exercise_id, role) VALUES ('cat_cow', 'Assess');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('cat_cow', 'Prepare');
INSERT INTO exercise_roles (exercise_id, role) VALUES ('cat_cow', 'Restore');

INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('cat_cow', 'Mobilise the spine through flexion and extension');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('cat_cow', 'Integrate diaphragmatic breathing with movement');
INSERT INTO exercise_objectives (exercise_id, objective) VALUES ('cat_cow', 'Assess spinal segmental awareness');

INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('cat_cow', 'Exhale to round — start from the tailbone and ripple up through the spine', 0);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('cat_cow', 'Inhale to extend — start from the breastbone and flow down', 1);
INSERT INTO exercise_teaching_cues (exercise_id, cue, sort_order) VALUES ('cat_cow', 'Keep the wrists under the shoulders and knees under the hips', 2);

INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('cat_cow', 'wrist_pain', 'Caution', 'Perform on fists or forearms to reduce wrist extension');
INSERT INTO exercise_contraindications (exercise_id, tag, severity, note) VALUES ('cat_cow', 'knee_pain', 'Caution', 'Add padding under the knees');
