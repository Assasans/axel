use serde_json::json;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

pub async fn story_list(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(json!({
      "storys": [
        {
          "user_story_id": 1,
          "story_type": 1,
          "story_id": 100001,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 2
        },
        {
          "user_story_id": 2,
          "story_type": 1,
          "story_id": 100101,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 1
        },
        {
          "user_story_id": 3,
          "story_type": 1,
          "story_id": 100102,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 4,
          "story_type": 1,
          "story_id": 100103,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 5,
          "story_type": 1,
          "story_id": 100104,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 6,
          "story_type": 1,
          "story_id": 100105,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 7,
          "story_type": 1,
          "story_id": 100106,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 8,
          "story_type": 1,
          "story_id": 100201,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 9,
          "story_type": 1,
          "story_id": 100202,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 10,
          "story_type": 1,
          "story_id": 100203,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 11,
          "story_type": 1,
          "story_id": 100204,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 12,
          "story_type": 1,
          "story_id": 100205,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 13,
          "story_type": 1,
          "story_id": 100206,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 14,
          "story_type": 1,
          "story_id": 100301,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 15,
          "story_type": 1,
          "story_id": 100302,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 16,
          "story_type": 1,
          "story_id": 100303,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 17,
          "story_type": 1,
          "story_id": 100304,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 18,
          "story_type": 1,
          "story_id": 100305,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 19,
          "story_type": 1,
          "story_id": 100306,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 20,
          "story_type": 1,
          "story_id": 100401,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 21,
          "story_type": 1,
          "story_id": 100402,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 22,
          "story_type": 1,
          "story_id": 100403,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 23,
          "story_type": 1,
          "story_id": 100404,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 24,
          "story_type": 1,
          "story_id": 100405,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 25,
          "story_type": 1,
          "story_id": 100406,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 26,
          "story_type": 1,
          "story_id": 100501,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 27,
          "story_type": 1,
          "story_id": 100502,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 28,
          "story_type": 1,
          "story_id": 100503,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 29,
          "story_type": 1,
          "story_id": 100504,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 30,
          "story_type": 1,
          "story_id": 100505,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 31,
          "story_type": 1,
          "story_id": 100506,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 0
        },
        {
          "user_story_id": 32,
          "story_type": 3,
          "story_id": 300101,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 2
        },
        {
          "user_story_id": 33,
          "story_type": 5,
          "story_id": 5230803,
          "force_release": false,
          "selections": [
            {
              "selection": []
            }
          ],
          "status": 2
        },
        {
          "user_story_id": 0,
          "story_type": 2,
          "story_id": 910742280,
          "force_release": true,
          "selections": [],
          "status": 0
        }
      ],
      "gettable_sp_story_member_id_list": []
    }))),
    false,
  ))
}
