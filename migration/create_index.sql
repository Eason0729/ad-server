CREATE INDEX idx_advertisement_end_at ON advertisement(end_at);
CREATE INDEX idx_advertisement_age_range ON advertisement(age_range);
CREATE INDEX idx_advertisement_cond ON advertisement(country, platform, gender);
CREATE INDEX idx_advertisement_country ON advertisement USING HASH (country);
CREATE INDEX idx_advertisement_platform ON advertisement USING HASH (platform);
CREATE INDEX idx_advertisement_gender ON advertisement USING HASH (gender);