# LARPd (Jobex) project plan

**Goal for this release.** Put a stable version of our short-term shift matching demo on a domain we buy. Add a real DeepSeek assistant that can answer questions about LARPd and help people use it. Then show the demo to employers and find out what would make it useful to them.

**Starting point.** LARPd already runs on [GitHub Pages](https://nexicv.github.io/jobex/). Workers browse shifts and express interest. Employers review candidates, and both sides can try the simulated chat, agreement, and feedback flow. Each visitor has their own demo data in the browser. [PR #7](https://github.com/nexicv/jobex/pull/7) contains the Ask LARPd panel and the Rust proxy, tested with a fake backend. The real DeepSeek connection and the domain purchase are still pending.

## 1. Backlog of tasks

| ID | Task | What we need to do | How we know it is done | Priority |
| --- | --- | --- | --- | --- |
| T01 | Connect DeepSeek through Oracle | Review PR #7 against current main. Deploy the Rust proxy to the Oracle VM, set up HTTPS, configure the API key and current token prices on the server, and run tests against real DeepSeek. Enable the assistant in the website after the tests pass. | Ask LARPd answers a question on the published site, finds a suitable shift, asks before submitting interest, and handles an API outage without breaking the site. | Must |
| T02 | Automate repetitive work with DeepSeek | Start by turning an employer's brief into a shift draft with the role, date, hours, pay, and skills. Reuse the existing draft tool and ask the employer for missing details. | A sample brief produces an editable draft. The employer checks it and confirms before publishing. | Should |
| T03 | Buy and connect a domain | Agree on a name, compare the first-year and renewal prices, choose who owns the registrar account, register the domain, and connect it to GitHub Pages. Set up a subdomain for the Oracle API. | The team knows who owns and renews the domain. The site loads over HTTPS, the alternate address redirects correctly, and the site reaches the API over HTTPS. | Must |
| T04 | Replace placeholder images | Start with the most visible shift cards. Use our own photos or images we have permission to use. Resize them for the web, record credits, and write useful alt text where needed. | The images show correctly on desktop and mobile. The team can trace each image to its source. | Should |
| T05 | Find interested employers | Shortlist cafés, event organizers, and other businesses that use short shifts. Ask what makes shifts hard to fill. Show them the demo and ask whether they would try a pilot. | We have a contact list, an outreach log, and a summary of the feedback we actually received. | Should |
| T06 | Promote the app | Prepare a short story or post and a poster with a QR code. Explain the value of LARPd in one sentence and point people to the final domain. | A test scan on a phone opens the right page. The material describes the demo accurately and gives people a way to leave feedback. | Should for stories and posters. Won't this time for a paid billboard. |
| T07 | Open directions from shift details | Add a Get directions button using the worksite address. Start with a Google Maps directions link. Consider an Apple Maps option or a copy-address button if time allows. | The link opens the intended destination on desktop and phone. A shift without an address shows a clear message. | Should for Google Maps. Could for extra options. |
| T08 | Improve the UI | Watch someone try to find a shift, express interest, and review an applicant. Fix confusing labels, navigation, and the Ask LARPd layout on smaller screens. | A tester finishes the main worker and employer journeys. The chat panel is still usable with the phone keyboard open. | Should |
| T09 | Fix site bugs | Run through onboarding, Discover, applications, hiring, chat, agreement previews, feedback, and reset. Fix broken links and bugs that stop a core flow. | Both roles can complete the demo script on a phone and a desktop using the published site. | Must |
| T10 | Secure the site and API | Keep secrets on Oracle, serve the API through Caddy over HTTPS, allow the new website origin, and set request and spending controls. Check that important actions still require confirmation. | The key stays out of the repo and browser, limits behave as expected, and the GameNest relay still works after deployment. | Must |
| T11 | Improve site performance | Measure how the site loads and responds on a phone or a throttled connection. Start with the biggest problem, such as heavy images, slow rendering, or an unresponsive chat panel. | We can show a before-and-after result using the same device or test settings. Issues that block the core demo go under T09. | Could |
| T12 | Plan a shared database | Identify what a pilot needs to share between users, including shifts, applications, and application status. Sketch who can see or edit each record and how we would reset a test dataset. | The team has a short data model and access plan for a future pilot. The class demo uses sample data in the browser. | Could for the design. Won't this time for production storage. |
| T13 | Review privacy, cookies, and policies | List what the site stores and sends, including browser storage, chat requests to Oracle and DeepSeek, and any server logs. Draft a privacy notice, a demo notice, and a contact route. Review consent requirements before adding optional analytics or tracking. | Users can find notices that describe what actually happens before they enter information. The team has reviewed the data and storage plan before inviting people to try it. | Must |

### How the domain and DeepSeek fit together

GitHub Pages hosts the website. The existing Oracle VM in Milan runs Caddy and the Rust proxy. The proxy calls DeepSeek's hosted API and streams the answer back to the website. The browser carries out approved actions using its own demo data. [LARPd assistant plan](https://github.com/nexicv/jobex/blob/feature/ai-assistant/assistant/PLAN.md) · [DeepSeek API documentation](https://api-docs.deepseek.com/)

1. Pick a domain name and agree on its first-year cost, renewal price, and who controls the account. Use **example.com** and **api.example.com** as placeholders until the team buys the domain.
2. Verify the domain in GitHub, configure DNS for the root and www addresses, then test HTTPS and the redirect from the alternate address. [GitHub Pages custom domains](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site)
3. Point the API subdomain at the Oracle VM. Use it in Caddy in place of the earlier DuckDNS name if the team agrees. Keep the Rust service on the VM's loopback interface. [Oracle deployment plan](https://github.com/nexicv/jobex/blob/feature/ai-assistant/assistant/PLAN.md#hosting-on-the-milan-vm) · [Caddy HTTPS](https://caddyserver.com/docs/automatic-https)
4. Emir enters the DeepSeek key on the VM. Add the new website origin to the proxy settings. Check the API health endpoint, run the real-model tests, then set the website's ASSISTANT_URL to the HTTPS API address. [Proxy documentation](https://github.com/nexicv/jobex/blob/feature/ai-assistant/assistant/proxy/README.md)
5. Test with sample names and conversations. Try shift search, a confirmed action, an off-topic question, and an unavailable API. Review what the privacy notice says about information sent to DeepSeek. [EU guidance on website privacy and cookies](https://europa.eu/youreurope/business/growing/digitalising/online-privacy/index_en.htm)

## 2. MoSCoW prioritization

The priorities apply to the **next class demo and employer preview**. [MoSCoW method](https://www.agilebusiness.org/resource/what-is-moscow-prioritization/)

- **Must have** covers the purchased domain, a real DeepSeek connection, working core flows, security, and the privacy review (T01, T03, T09, T10, T13). All five are needed to call this release ready.
- **Should have** covers AI-assisted shift drafting, real images, employer outreach, basic promotion, Google Maps directions, and UI improvements (T02, T04–T08). If time is short, start with one poster and the images on the cards people see first. [Google Maps directions links](https://developers.google.com/maps/documentation/urls/get-started)
- **Could have** covers extra performance work, a database design sketch, and a second maps option (T11 and parts of T12 and T07). Start these once the Must tasks are working and there is time to test.
- **Won't have this time** covers a paid billboard and a production database for real applicants (parts of T06 and T12). Employer feedback can help us decide whether either is worth funding later.

## 3. RACI matrix

**R** does the work. **A** owns the result and gives final approval. **C** gives input before a decision. **I** receives status updates. Each row has one accountable role. **A/R** means the same role does the work and owns the result. [RACI method](https://www.pmi.org/pmiheadless/home/blog/blog-posts/2026/07/27/15/54/stakeholder-management-raci)

Emir owns DeepSeek and Oracle. Olaf owns the domain. The other assignments below are proposed for the team. Each person can do their assigned work with normal project access and free tools. Emir handles paid DeepSeek API calls and runs the live tests when the team needs them. Filip, Doğukan, Berat, and Olaf can use the fake backend or the published demo for testing without individual AI subscriptions. [Fake backend setup](https://github.com/nexicv/jobex/blob/feature/ai-assistant/assistant/dev/README.md)

| Person | What they own |
| --- | --- |
| Emir | DeepSeek and Oracle Cloud setup, Caddy and API deployment, API key and spending controls, security checks, and live DeepSeek tests. |
| Olaf | Pick and buy the domain after the team agrees on the budget, manage DNS and renewal, verify the GitHub Pages connection, and own the privacy and demo notices. Coordinate the API DNS record with Emir. |
| Filip | Replace placeholder images and track credits, improve the layout on mobile and desktop, and measure whether the site loads and responds better. |
| Doğukan | Add the directions link, build and test the shift-draft flow with sample data, fix site bugs, and run the release checklist for both roles. Sketch the shared database if time allows. |
| Berat | Find employers to interview, track feedback, and prepare stories, a poster, and a QR code for the demo. Draft the plain-language notices with Olaf. |

| Task | Emir | Olaf | Filip | Doğukan | Berat |
| --- | --- | --- | --- | --- | --- |
| T01 DeepSeek and Oracle | A/R | C | C | C | I |
| T02 AI-assisted shift draft | R | I | C | A/R | C |
| T03 Domain purchase and setup | C | A/R | I | C | I |
| T04 Real images | I | I | A/R | C | C |
| T05 Find employers | I | I | C | C | A/R |
| T06 Stories and poster | I | C | R | I | A/R |
| T07 Directions link | I | I | C | A/R | C |
| T08 UI improvements | C | I | A/R | R | C |
| T09 Site bugs and regression checks | C | I | R | A/R | C |
| T10 Security | A/R | C | I | R | I |
| T11 Performance | C | I | A/R | R | I |
| T12 Database design | C | I | I | A/R | C |
| T13 Privacy, cookies, and policies | R | A/R | C | C | R |

For T02, Doğukan prepares the draft flow and test cases. Emir runs the tests that use the paid API. For T13, Emir documents what Oracle and DeepSeek receive, Berat drafts the notices, and Olaf approves them for publication.

## 4. Project milestones

| Milestone | Target | Complete when |
| --- | --- | --- |
| M1. Scope agreed and domain purchased | End of Week 1 | The team confirms the assignments and Must items, the domain is registered, and core bugs are recorded and assigned. |
| M2. Website live on the new domain | End of Week 2 | GitHub Pages serves the custom address over HTTPS, redirects work, and the worker and employer journeys pass a first check. |
| M3. Real AI works through Oracle | End of Week 3 | The API hostname and health check work. Real DeepSeek tests cover shift search, a confirmation, an off-topic request, and an error. A visitor can use Ask LARPd from the custom domain. |
| M4. Demo ready for class and employer feedback | End of Week 4 | Core flows pass on desktop and phone, the notices are visible, and the team has rehearsed. Outreach is logged and any feedback received has been turned into follow-up tasks. |

## 5. Project timeline

The four weeks are a proposed schedule from kickoff. Add calendar dates once we know the class deadline and team availability.

| Week | Technical work | Design and outreach | Checkpoint |
| --- | --- | --- | --- |
| 1 | Review the AI branch, record existing bugs, and confirm access to the repo and VM. Agree on a domain, registrar, and budget, then register it. | Confirm the named assignments, list the images to replace, shortlist employers, and draft the demo script for both roles. | M1 |
| 2 | Connect GitHub Pages to the domain, verify HTTPS, fix release blockers, and prepare the Oracle configuration. Add Google Maps directions if core flows are stable. | Start the privacy inventory and notice. Test the phone layout and replace the most visible placeholders if time allows. | M2 |
| 3 | Deploy the Oracle proxy and Caddy, configure the key and prices, and run real DeepSeek tests. Enable Ask LARPd after it passes. Try AI-assisted shift drafting if the core assistant is stable. | Test the chat panel on a phone, finish the demo script, and prepare the story and poster. | M3 |
| 4 | Recheck the full site, spend controls, and API fallback. Fix demo blockers and keep time for DNS or API delays. | Publish the approved notices, test the QR code, rehearse the class demo, and invite shortlisted employers to review it. | M4 |

Domain and DNS work can run in parallel with the AI branch review. Ask LARPd depends on the Oracle API passing its tests and accepting requests from the new website. The poster and QR code depend on the final address. Employer research can start in Week 1, with demo invitations after the site and notices are ready. If the rollout slips, use the existing GitHub Pages demo for class and report the domain and live AI milestones as pending.
