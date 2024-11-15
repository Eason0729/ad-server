import {
  randomIntBetween,
  randomItem,
} from "https://jslib.k6.io/k6-utils/1.2.0/index.js";
import http from "k6/http";
import { sleep } from "k6";
import { randomString } from "https://jslib.k6.io/k6-utils/1.2.0/index.js";

export const options = {
  scenarios: {
    contacts: {
      executor: "ramping-vus",
      startVUs: 100,
      stages: [
        { target: 200, duration: "10s" },
      ],
    },
  },
  // A number specifying the number of VUs to run concurrently.
  // vus: 10,
  // A string specifying the total duration of the test run.
  // duration: '30s',

  // The following section contains configuration options for execution of this
  // test script in Grafana Cloud.
  //
  // See https://grafana.com/docs/grafana-cloud/k6/get-started/run-cloud-tests-from-the-cli/
  // to learn about authoring and running k6 test scripts in Grafana k6 Cloud.
  //
  // cloud: {
  //   // The ID of the project to which the test is assigned in the k6 Cloud UI.
  //   // By default tests are executed in default project.
  //   projectID: "",
  //   // The name of the test in the k6 Cloud UI.
  //   // Test runs with the same name will be grouped.
  //   name: "script.js"
  // },

  // Uncomment this section to enable the use of Browser API in your tests.
  //
  // See https://grafana.com/docs/k6/latest/using-k6-browser/running-browser-tests/ to learn more
  // about using Browser API in your test scripts.
  //
  // scenarios: {
  //   // The scenario name appears in the result summary, tags, and so on.
  //   // You can give the scenario any name, as long as each name in the script is unique.
  //   ui: {
  //     // Executor is a mandatory parameter for browser-based tests.
  //     // Shared iterations in this case tells k6 to reuse VUs to execute iterations.
  //     //
  //     // See https://grafana.com/docs/k6/latest/using-k6/scenarios/executors/ for other executor types.
  //     executor: 'shared-iterations',
  //     options: {
  //       browser: {
  //         // This is a mandatory parameter that instructs k6 to launch and
  //         // connect to a chromium-based browser, and use it to run UI-based
  //         // tests.
  //         type: 'chromium',
  //       },
  //     },
  //   },
  // }
};

// The function that defines VU logic.
//
// See https://grafana.com/docs/k6/latest/examples/get-started-with-k6/ to learn more
// about authoring k6 scripts.
//
const genders = ["male", "female"];
const countries = ["US", "TW", "GB", "AU", "FR", "DE"];
const platforms = ["android", "ios", "web", "desktop", "smarttv"];
export default function () {
  const title = randomString(16, "aeioubcdfghijpqrstuv");
  const from_age = randomIntBetween(1, 10);
  const to_age = from_age + randomIntBetween(1, 95);
  const gender = randomItem(genders);
  const country = randomItem(countries);
  const platform = randomItem(platforms);

  for (let i = 0; i < 25; i++) {
    http.post(
      `http://ad-server.local/ad`,
      JSON.stringify({
        title,
        country,
        to_age,
        from_age,
        gender,
        platform,
        end_at: "2026-08-15T17:41:18.106108",
      }),
      {
        headers: { "Content-Type": "application/json" },
      },
    );
  }

  sleep(1);
}
