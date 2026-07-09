var SEED_DATA = (() => {
    const strategies = [
        {
            movement_experience: 'ShoulderFreedom',
            primary_focus: 'Shoulder',
            secondary_focus: 'Thoracic',
            emphasis: { Shoulder: 40, Thoracic: 22, BreathCore: 14, Spine: 10, Hip: 8, Legs: 4, Balance: 2 },
            objectives: ['Increase overhead reach range', 'Improve scapular stability', 'Release thoracic extension'],
            explanation_template: 'Prioritise shoulder range, scapular control, and thoracic extension so reaching feels easier without rib flare.'
        },
        {
            movement_experience: 'HappyHips',
            primary_focus: 'Hip',
            secondary_focus: 'Legs',
            emphasis: { Hip: 36, Legs: 24, Spine: 14, BreathCore: 12, Balance: 8, Shoulder: 4, Thoracic: 2 },
            objectives: ['Improve hip flexion and extension range', 'Strengthen glute activation', 'Improve single-leg stability'],
            explanation_template: 'Prioritise hip mobility, glute activation, and leg alignment so squatting and single-leg stance feel clearer.'
        },
        {
            movement_experience: 'SpineReset',
            primary_focus: 'Spine',
            secondary_focus: 'BreathCore',
            emphasis: { Spine: 34, BreathCore: 24, Thoracic: 18, Hip: 10, Shoulder: 8, Legs: 4, Balance: 2 },
            objectives: ['Restore spinal segmental mobility', 'Improve diaphragmatic breathing', 'Reduce spinal compression patterns'],
            explanation_template: 'Prioritise spinal articulation, rotation, and breath-led core support so posture feels decompressed and organised.'
        }
    ];

    const modifiers = [
        mod('thoracic_stiffness', ['thoracic', 'stiff', 'rounded'], { Thoracic: 7, Spine: 3 }, [], ['Release thoracic extension'], 'Thoracic stiffness was observed, so thoracic mobility receives extra priority.'),
        mod('limited_overhead_reach', ['overhead', 'reach', 'arm'], { Shoulder: 6, Thoracic: 3 }, [], ['Increase overhead reach range'], 'Limited overhead reach was observed, so shoulder range and rib control are prioritised.'),
        mod('desk_posture', ['desk', 'office', 'sitting'], { Thoracic: 5, Shoulder: 4, BreathCore: 3 }, ['Counter forward-head and rounded-shoulder posture'], ['Release thoracic extension'], 'Desk posture patterns were observed, so the plan counters forward head, rounded shoulders, and shallow breath.'),
        mod('hip_tightness', ['hip tight', 'hip stiffness', 'tight hips'], { Hip: 7, Legs: -2 }, [], ['Improve hip flexion and extension range'], 'Hip tightness was observed, so hip flexor, rotator, and glute work receive extra priority.'),
        mod('limited_squat', ['squat', 'depth'], { Hip: 5, Legs: 4 }, [], ['Improve hip flexion and extension range'], 'Limited squat depth was observed, so hip mobility and leg alignment receive extra priority.'),
        mod('low_back_tension', ['low back', 'lumbar', 'back tension'], { Spine: 6, BreathCore: 5 }, ['Create spinal decompression before load'], ['Reduce spinal compression patterns'], 'Low-back tension was observed, so decompression and breath-led core support receive extra priority.'),
        mod('limited_rotation', ['rotation', 'rotate', 'twist'], { Thoracic: 5, Spine: 5 }, [], ['Restore spinal segmental mobility'], 'Limited rotation was observed, so thoracic and spinal rotation receive extra priority.'),
        mod('weak_core', ['weak core', 'core connection', 'stability'], { BreathCore: 7, Balance: -2 }, ['Activate deep stabilisers before larger movement'], ['Improve diaphragmatic breathing'], 'Weak core connection was observed, so deep stabiliser activation is introduced earlier.'),
        mod('balance_concern', ['balance', 'unstable'], { Balance: 5, Legs: -2 }, [], ['Improve single-leg stability'], 'Balance uncertainty was observed, so balance challenge is introduced gradually.'),
        mod('knee_pain', ['knee'], { Legs: -6, Hip: 4 }, ['Protect knee joints by reducing loaded flexion'], [], 'Knee sensitivity was reported, so loaded knee flexion is reduced and hip mechanics receive extra priority.'),
        mod('wrist_pain', ['wrist'], { Shoulder: -3, BreathCore: 2 }, ['Avoid sustained weight-bearing through wrists'], [], 'Wrist sensitivity was reported, so loaded wrist positions are avoided or regressed.')
    ];

    const benchmarks = [
        bench('overhead_reach', 'ShoulderFreedom', 'Overhead Reach Check', 'Stand tall and reach both arms overhead. Notice rib flare, shoulder hiking, and left-right range.', ['Ribs flare before arms reach vertical', 'Shoulders hike or pinch near the ears', 'One arm arrives later than the other']),
        bench('wall_slide', 'ShoulderFreedom', 'Wall Slide Check', 'Stand with back near a wall and slide arms upward while keeping ribs quiet. Notice where motion changes quality.', ['Lower back arches to find range', 'Elbows or wrists drift away from wall']),
        bench('deep_squat', 'HappyHips', 'Deep Squat Check', 'Lower into a comfortable squat. Notice depth, heel contact, knee tracking, and torso angle.', ['Heels lift or knees collapse inward', 'Torso falls forward early']),
        bench('single_leg_stance', 'HappyHips', 'Single-Leg Stance Check', 'Balance on one leg for up to 30 seconds. Notice pelvis level, foot pressure, and sway.', ['Pelvis drops or foot grips hard', 'Balance cannot settle within 10 seconds']),
        bench('standing_roll_down', 'SpineReset', 'Standing Roll Down Check', 'Roll down one segment at a time. Notice where the spine moves freely and where it skips.', ['Movement skips through thoracic or lumbar spine', 'Breath holds during flexion']),
        bench('seated_rotation', 'SpineReset', 'Seated Rotation Check', 'Sit tall and rotate right and left. Notice range, breath, and side-to-side difference.', ['Hips shift instead of thorax rotating', 'One direction feels blocked or compressed'])
    ];

    const exercises = [
        ex('footwork', 'Footwork', 'Reformer', 'Beginner', 'IntermediateAdvanced', 2, 7, ['Prepare', 'Prime'], ['Legs', 'Hip', 'BreathCore'], ['HappyHips', 'SpineReset'], ['Improve hip flexion and extension range', 'Strengthen glute activation', 'Build lower-body alignment'], ['Press through the whole foot and keep knees tracking over toes.', 'Exhale to press, inhale to return with control.'], 'Reduce spring load and range.', 'Add single-leg variations.'),
        ex('pelvic_curl', 'Pelvic Curl', 'Reformer', 'Beginner', 'IntermediateAdvanced', 2, 6, ['Prepare', 'Prime', 'Restore'], ['Spine', 'Hip', 'BreathCore'], ['SpineReset', 'HappyHips'], ['Restore spinal segmental mobility', 'Strengthen glute activation', 'Reduce spinal compression patterns'], ['Roll one segment at a time and keep the ribs heavy.', 'Feel hamstrings and glutes support the lift.'], 'Bridge only halfway.', 'Add arm reach or slower tempo.'),
        ex('feet_in_straps', 'Feet in Straps', 'Reformer', 'BeginnerIntermediate', 'Advanced', 3, 8, ['Prime', 'Integrate'], ['Hip', 'Legs', 'BreathCore'], ['HappyHips'], ['Improve hip flexion and extension range', 'Improve pelvic control', 'Lengthen posterior chain'], ['Move the legs from the hips while the pelvis stays quiet.', 'Keep the back of the ribs wide on the carriage.'], 'Use smaller arcs.', 'Add coordination or longer lever.'),
        ex('arms_in_straps', 'Arms in Straps', 'Reformer', 'BeginnerIntermediate', 'Advanced', 3, 7, ['Prime', 'Integrate'], ['Shoulder', 'BreathCore'], ['ShoulderFreedom'], ['Improve scapular stability', 'Increase overhead reach range', 'Improve rib control'], ['Reach from the back of the shoulders, not the neck.', 'Keep ribs quiet as the arms move.'], 'Bend elbows and reduce spring.', 'Add hundred prep pattern.'),
        ex('pulling_straps', 'Pulling Straps', 'Reformer', 'Intermediate', 'Advanced', 4, 6, ['Prime', 'Challenge'], ['Shoulder', 'Thoracic', 'Spine'], ['ShoulderFreedom', 'SpineReset'], ['Release thoracic extension', 'Improve scapular stability', 'Strengthen posterior shoulder'], ['Lengthen before lifting the chest.', 'Draw shoulder blades down and wide.'], 'Keep chest low and reduce range.', 'Add T pull or longer hold.'),
        ex('short_spine_prep', 'Short Spine Prep', 'Reformer', 'Intermediate', 'Advanced', 4, 7, ['Integrate', 'Restore'], ['Spine', 'Hip', 'BreathCore'], ['SpineReset', 'HappyHips'], ['Restore spinal segmental mobility', 'Improve hip mobility', 'Improve diaphragmatic breathing'], ['Fold at the hips before articulating the spine.', 'Control the return one segment at a time.'], 'Keep pelvis grounded and use frog pattern.', 'Move toward full short spine.'),
        ex('elephant', 'Elephant', 'Reformer', 'BeginnerIntermediate', 'Advanced', 3, 6, ['Prime', 'Challenge'], ['Spine', 'Shoulder', 'Legs'], ['SpineReset', 'ShoulderFreedom'], ['Reduce spinal compression patterns', 'Release thoracic extension', 'Lengthen posterior chain'], ['Lift the ribs away from the wrists.', 'Move the carriage from the hips and breath.'], 'Bend knees and keep range small.', 'Add single-leg variation.', [{ tag: 'high_blood_pressure', severity: 'HardExclude', note: 'Avoid loaded inverted hinge for this group.' }]),
        ex('eves_lunge', "Eve's Lunge", 'Reformer', 'BeginnerIntermediate', 'Advanced', 3, 6, ['Prepare', 'Transfer'], ['Hip', 'Legs', 'Spine'], ['HappyHips'], ['Improve hip flexion and extension range', 'Open hip flexors', 'Transfer hip extension to standing'], ['Reach the back thigh long while keeping the pelvis level.', 'Breathe into the front of the hip.'], 'Use hands on frame and smaller range.', 'Add rotation or arm reach.'),
        ex('chair_footwork', 'Chair Footwork', 'Chair', 'Beginner', 'IntermediateAdvanced', 2, 7, ['Prepare', 'Prime'], ['Legs', 'Hip'], ['HappyHips'], ['Build lower-body alignment', 'Strengthen glute activation', 'Improve knee tracking'], ['Press evenly through both feet and keep the pelvis tall.', 'Control the pedal return without dropping.'], 'Use lighter spring and smaller knee bend.', 'Add heel/toe variations.', [{ tag: 'knee_pain', severity: 'Caution', note: 'Use smaller knee bend and lighter spring.' }]),
        ex('seated_push_down', 'Seated Push Down', 'Chair', 'Beginner', 'IntermediateAdvanced', 2, 6, ['Prepare', 'Prime'], ['Shoulder', 'BreathCore'], ['ShoulderFreedom'], ['Improve scapular stability', 'Increase overhead reach range', 'Improve diaphragmatic breathing'], ['Grow tall as the pedal moves.', 'Keep collarbones wide and neck easy.'], 'Use two hands and light spring.', 'Add single-arm version.', [{ tag: 'wrist_pain', severity: 'Caution', note: 'Use lighter spring and neutral wrist.' }]),
        ex('standing_push_down', 'Standing Push Down', 'Chair', 'BeginnerIntermediate', 'Advanced', 3, 6, ['Prime', 'Challenge'], ['Shoulder', 'BreathCore', 'Balance'], ['ShoulderFreedom'], ['Improve scapular stability', 'Improve lateral core control', 'Challenge standing posture'], ['Press from the shoulder blade, not the wrist.', 'Stand tall and keep weight even between feet.'], 'Use two hands or reduce range.', 'Add contralateral reach.', [{ tag: 'wrist_pain', severity: 'HardExclude', note: 'Avoid standing pedal press with loaded wrist pressure.' }]),
        ex('swan_on_chair', 'Swan on Chair', 'Chair', 'BeginnerIntermediate', 'Advanced', 3, 6, ['Prime', 'Integrate'], ['Spine', 'Thoracic', 'Shoulder'], ['SpineReset', 'ShoulderFreedom'], ['Release thoracic extension', 'Restore spinal segmental mobility', 'Open anterior shoulders'], ['Initiate extension from the upper back.', 'Keep the lower back long.'], 'Keep extension small.', 'Add longer hold or leg reach.', [{ tag: 'low_back_pain', severity: 'Caution', note: 'Keep extension small and bias upper thoracic movement.' }]),
        ex('mermaid_chair', 'Mermaid on Chair', 'Chair', 'Beginner', 'IntermediateAdvanced', 2, 6, ['Prepare', 'Integrate'], ['Spine', 'Thoracic', 'BreathCore'], ['SpineReset', 'ShoulderFreedom'], ['Restore spinal segmental mobility', 'Improve diaphragmatic breathing', 'Release thoracic extension'], ['Anchor both sit bones before side bending.', 'Breathe into the opening side ribs.'], 'Keep both sit bones grounded.', 'Add rotation.'),
        ex('step_up_prep', 'Step Up Prep', 'Chair', 'Intermediate', 'Advanced', 4, 6, ['Challenge', 'Transfer'], ['Legs', 'Hip', 'Balance'], ['HappyHips'], ['Improve single-leg stability', 'Strengthen glute activation', 'Build functional stepping strength'], ['Drive through the standing heel and keep pelvis level.', 'Use the hands only as much as needed.'], 'Use hand support and smaller pedal travel.', 'Add arm reach.', [{ tag: 'knee_pain', severity: 'RequireRegression', note: 'Use hand support and reduce pedal travel.' }]),
        ex('standing_leg_pump', 'Standing Leg Pump', 'Chair', 'Intermediate', 'Advanced', 4, 6, ['Prime', 'Challenge'], ['Hip', 'Legs', 'Balance'], ['HappyHips'], ['Strengthen glute activation', 'Improve single-leg stability', 'Build lower-body endurance'], ['Press through the heel and keep the torso tall.', 'Control the pedal return with the back of the leg.'], 'Hold chair and reduce spring.', 'Add upright arm pattern.', [{ tag: 'knee_pain', severity: 'Caution', note: 'Reduce range and spring resistance.' }]),
        ex('seated_rotation_chair', 'Seated Rotation on Chair', 'Chair', 'BeginnerIntermediate', 'Advanced', 3, 5, ['Prime', 'Integrate', 'Restore'], ['Spine', 'Thoracic', 'BreathCore'], ['SpineReset'], ['Restore spinal segmental mobility', 'Improve diaphragmatic breathing', 'Release thoracic rotation'], ['Rotate from the ribs while the pelvis stays grounded.', 'Exhale into the turn and inhale back to centre.'], 'Reduce rotation range.', 'Add arm arc.'),
        ex('breath_reset', 'Breath Reset', 'Mat', 'Beginner', 'Advanced', 1, 4, ['Assess', 'Prepare', 'Restore'], ['BreathCore', 'Spine'], ['SpineReset', 'ShoulderFreedom', 'HappyHips'], ['Improve diaphragmatic breathing', 'Reduce spinal compression patterns', 'Settle attention'], ['Let the inhale widen the side and back ribs.', 'Exhale without gripping the throat or jaw.'], 'Use supported knees.', 'Add arm float.'),
        ex('standing_roll_down_ctx', 'Standing Roll Down', 'Standing', 'Beginner', 'Advanced', 1, 4, ['Assess', 'Prepare', 'Restore'], ['Spine', 'BreathCore'], ['SpineReset', 'HappyHips'], ['Restore spinal segmental mobility', 'Reduce spinal compression patterns', 'Assess posterior chain length'], ['Nod the head and roll down one segment at a time.', 'Soften knees if the back line pulls.'], 'Bend knees and reduce range.', 'Slow tempo or close eyes.'),
        ex('arm_raise_ctx', 'Arm Raise', 'Standing', 'Beginner', 'Advanced', 1, 3, ['Assess', 'Prepare'], ['Shoulder', 'Thoracic'], ['ShoulderFreedom'], ['Increase overhead reach range', 'Improve scapular stability', 'Assess rib compensation'], ['Reach the fingertips up without flaring the ribs.', 'Keep shoulders broad and neck easy.'], 'Use smaller range.', 'Add light hand weights.'),
        ex('cat_cow', 'Cat-Cow', 'Mat', 'Beginner', 'Advanced', 1, 5, ['Assess', 'Prepare', 'Restore'], ['Spine', 'BreathCore'], ['SpineReset'], ['Restore spinal segmental mobility', 'Improve diaphragmatic breathing', 'Reduce spinal compression patterns'], ['Start the movement from the tail and ripple through the spine.', 'Match flexion and extension with breath.'], 'Perform seated on a chair.', 'Add limb reach.', [{ tag: 'wrist_pain', severity: 'RequireRegression', note: 'Use fists, forearms, or seated version.' }]),
        ex('standing_rotation_ctx', 'Standing Rotation', 'Standing', 'BeginnerIntermediate', 'Advanced', 2, 5, ['Prepare', 'Integrate', 'Transfer'], ['Thoracic', 'Spine', 'BreathCore'], ['SpineReset', 'ShoulderFreedom'], ['Restore spinal segmental mobility', 'Release thoracic rotation', 'Integrate upper and lower body'], ['Rotate from the ribcage and keep hips steady.', 'Let the eyes lead the turn.'], 'Reduce range.', 'Add single-leg stance.'),
        ex('squat_reach_ctx', 'Squat and Reach', 'Standing', 'BeginnerIntermediate', 'Advanced', 2, 5, ['Integrate', 'Transfer'], ['Hip', 'Legs', 'Shoulder'], ['HappyHips', 'ShoulderFreedom'], ['Improve hip flexion and extension range', 'Build lower-body alignment', 'Increase overhead reach range'], ['Sit back into the hips and reach through the arms.', 'Keep knees tracking and chest broad.'], 'Reduce squat depth.', 'Add rotation at the bottom.', [{ tag: 'knee_pain', severity: 'Caution', note: 'Limit squat depth to pain-free range.' }]),
        ex('single_leg_balance_ctx', 'Single-Leg Balance', 'Standing', 'BeginnerIntermediate', 'Advanced', 2, 4, ['Transfer'], ['Balance', 'Hip', 'Legs'], ['HappyHips'], ['Improve single-leg stability', 'Strengthen glute activation', 'Challenge balance control'], ['Root through the standing foot and level the pelvis.', 'Keep the eyes on one steady point.'], 'Use wall support.', 'Add head turns.', [{ tag: 'ankle_instability', severity: 'Caution', note: 'Use wall support and reduce challenge.' }]),
        ex('hip_hinge_ctx', 'Hip Hinge', 'Standing', 'BeginnerIntermediate', 'Advanced', 2, 5, ['Integrate', 'Transfer'], ['Hip', 'Legs', 'Spine'], ['HappyHips', 'SpineReset'], ['Strengthen glute activation', 'Reduce spinal compression patterns', 'Teach hip-dominant movement'], ['Send hips back while the spine stays long.', 'Feel hamstrings load before returning upright.'], 'Use dowel feedback.', 'Add single-leg hinge.', [{ tag: 'low_back_pain', severity: 'RequireRegression', note: 'Use dowel feedback and reduce range.' }])
    ];

    function mod(id, keywords, emphasis, added_objectives, preferred_objectives, explanation_fragment) {
        return { id, keywords, emphasis, added_objectives, preferred_objectives, explanation_fragment };
    }

    function bench(id, movement_experience, name, instruction, watch_points) {
        return { id, movement_experience, name, instruction, watch_points };
    }

    function ex(id, name, equipment, min_level, max_level, difficulty, default_duration_minutes, roles, systems, experiences, objectives, cues, regression, progression, contraindications = []) {
        return {
            id,
            name,
            description: name,
            equipment,
            min_level,
            max_level,
            difficulty,
            default_duration_minutes,
            roles,
            systems,
            experiences,
            objectives,
            cues,
            regression,
            progression,
            contraindications
        };
    }

    return { strategies, modifiers, benchmarks, exercises };
})();
