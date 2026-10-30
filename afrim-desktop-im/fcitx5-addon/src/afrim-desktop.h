/*
 * AfrimDesktop fcitx5 addon
 */

#ifndef FCITX5_AFRIM_DESKTOP_AFRIM_DESKTOP_H
#define FCITX5_AFRIM_DESKTOP_AFRIM_DESKTOP_H

#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/candidatelist.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/instance.h>

// Include the Rust FFI header
#include "../../include/afrim_desktop.h"

namespace fcitx {

class AfrimDesktopState;
class AfrimDesktopEngine;

// Candidate word class
class AfrimDesktopCandidateWord : public CandidateWord {
public:
    AfrimDesktopCandidateWord(AfrimDesktopEngine* engine, Text text, int index,
                         const std::string& annotation = "");
    void select(InputContext* inputContext) const override;

private:
    AfrimDesktopEngine* engine_;
    int index_;
};

// Candidate list class
class AfrimDesktopCandidateList : public CommonCandidateList {
public:
    AfrimDesktopCandidateList(AfrimDesktopEngine* engine, InputContext* ic);
    void updateCandidates(::AfrimDesktopEngine* rustEngine);

private:
    AfrimDesktopEngine* engine_;
    InputContext* ic_;
};

// Per-input-context state
class AfrimDesktopState : public InputContextProperty {
public:
    AfrimDesktopState(AfrimDesktopEngine* engine, InputContext* ic);
    ~AfrimDesktopState() override;

    void keyEvent(KeyEvent& keyEvent);
    void reset();
    void updateUI();

    ::AfrimDesktopEngine* rustEngine() { return rustEngine_; }

private:
    AfrimDesktopEngine* engine_;
    InputContext* ic_;
    ::AfrimDesktopEngine* rustEngine_{nullptr};
    bool engineInitialized_{false};
};

// Main engine class
class AfrimDesktopEngine : public InputMethodEngineV3 {
public:
    AfrimDesktopEngine(Instance* instance);
    ~AfrimDesktopEngine() override;

    void keyEvent(const InputMethodEntry& entry, KeyEvent& keyEvent) override;
    void reset(const InputMethodEntry& entry, InputContextEvent& event) override;
    void activate(const InputMethodEntry& entry, InputContextEvent& event) override;
    void deactivate(const InputMethodEntry& entry, InputContextEvent& event) override;

    Instance* instance() { return instance_; }

    void selectCandidate(InputContext* ic, int index);

    auto& factory() { return factory_; }

private:
    Instance* instance_;
    FactoryFor<AfrimDesktopState> factory_;
};

class AfrimDesktopEngineFactory : public AddonFactory {
public:
    AddonInstance* create(AddonManager* manager) override {
        return new AfrimDesktopEngine(manager->instance());
    }
};

}  // namespace fcitx

#endif  // FCITX5_AFRIM_DESKTOP_AFRIM_DESKTOP_H
