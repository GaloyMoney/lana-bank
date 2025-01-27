{% macro create_udf_loan_ytm_from_price() %}

CREATE OR REPLACE FUNCTION {{target.schema}}.udf_loan_ytm_from_price (interest_rate FLOAT64, times ARRAY<FLOAT64>, cash_flows ARRAY<FLOAT64>)
RETURNS FLOAT64
LANGUAGE js
AS r"""
  const loan_pv = function loan_pv(interest_rate, times, cash_flows) {
      if (times.length != cash_flows.length) {
          return NaN;
      }

      var pv = 0;
      for (var i = 0; i < times.length; i++) {
          pv += cash_flows[i] / ((1 + interest_rate) ** times[i]);
      }

      return pv;
  };

  const loan_ytm = function loan_ytm(interest_rate, times, cash_flows) {
      if (times.length != cash_flows.length) {
          return NaN;
      }

      const ACCURACY = 0.00005;
      const MAX_ITERATIONS = 200;

      var bottom = 0, top = 1;

      var pv = loan_pv(interest_rate, times, cash_flows);

      while (loan_pv(top, times, cash_flows) > pv) { top = top * 2; }
      var ytm = 0.5 * (top + bottom);
      for (i = 0; i < MAX_ITERATIONS; i++) {
          var diff = loan_pv(ytm, times, cash_flows) - pv;
          if (Math.abs(diff) < ACCURACY) { return ytm; }
          if (diff > 0) { bottom = ytm; }
          else { top = ytm; }
          ytm = 0.5 * (top + bottom);
      }

      return ytm;
  };

  const loan_ytm_from_price = function loan_ytm_from_price(pv, times, cash_flows) {
      if (times.length != cash_flows.length) {
          return NaN;
      }

      const ACCURACY = 0.00005;
      const MAX_ITERATIONS = 200;

      var bottom = 0, top = 1;

      while (loan_pv(top, times, cash_flows) > pv) { top = top * 2; }
      var ytm = 0.5 * (top + bottom);
      for (i = 0; i < MAX_ITERATIONS; i++) {
          var diff = loan_pv(ytm, times, cash_flows) - pv;
          if (Math.abs(diff) < ACCURACY) { return ytm; }
          if (diff > 0) { bottom = ytm; }
          else { top = ytm; }
          ytm = 0.5 * (top + bottom);
      }

      return ytm;
  };

  const loan_duration = function loan_duration(interest_rate, times, cash_flows) {
      if (times.length != cash_flows.length) {
          return NaN;
      }

      var pv = loan_pv(interest_rate, times, cash_flows);

      var duration_sum = 0;
      for (var i = 0; i < times.length; i++) {
          duration_sum += times[i] * cash_flows[i] / ((1 + interest_rate) ** times[i]);
      }

      return duration_sum / pv;
  };

  const loan_mac_duration = function loan_mac_duration(interest_rate, times, cash_flows) {
      if (times.length != cash_flows.length) {
          return NaN;
      }

      var ytm = loan_ytm(interest_rate, times, cash_flows);
      var duration = loan_duration(ytm, times, cash_flows);

      return duration;
  };

  const loan_mod_duration = function loan_mod_duration(interest_rate, times, cash_flows) {
      if (times.length != cash_flows.length) {
          return NaN;
      }

      var ytm = loan_ytm(interest_rate, times, cash_flows);
      var duration = loan_duration(ytm, times, cash_flows);

      return duration / (1 + ytm);
  };

  const loan_convexity = function loan_convexity(interest_rate, times, cash_flows) {
      if (times.length != cash_flows.length) {
          return NaN;
      }

      var pv = loan_pv(interest_rate, times, cash_flows);

      var convex_sum = 0;
      for (var i = 0; i < times.length; i++) {
          convex_sum += cash_flows[i] * times[i] * (times[i] + 1) / ((1 + interest_rate) ** times[i]);
      }

      return convex_sum / ((1 + interest_rate) ** 2) / pv;
  };

  const loan_pv_delta_on_interest_rate_delta = function loan_pv_delta_on_interest_rate_delta(interest_rate, times, cash_flows, interest_rate_delta) {
      if (times.length != cash_flows.length) {
          return NaN;
      }

      var pv = loan_pv(interest_rate, times, cash_flows);
      var duration = loan_mod_duration(interest_rate, times, cash_flows);
      var delta_percent = -duration * interest_rate_delta;

      return delta_percent * pv;
  };

  const loan_pv_delta_on_interest_rate_delta_with_convex = function loan_pv_delta_on_interest_rate_delta_with_convex(interest_rate, times, cash_flows, interest_rate_delta) {
      if (times.length != cash_flows.length) {
          return NaN;
      }

      var pv = loan_pv(interest_rate, times, cash_flows);
      var duration = loan_mod_duration(interest_rate, times, cash_flows);
      var convexity = loan_convexity(interest_rate, times, cash_flows);
      var delta_percent = -duration * interest_rate_delta;
      delta_percent += (convexity / 2) * (interest_rate_delta ** 2);

      return delta_percent * pv;
  };

  return loan_ytm_from_price(interest_rate, times, cash_flows);
"""

{% endmacro %}
